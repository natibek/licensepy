use crate::utils::{Config, read_config};
use colored::Colorize;
use log::debug;
use rayon::prelude::*;
use regex::Regex;
use std::fs::{DirEntry, File, read_dir};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::exit;

const COMMENT: &str = "#";
const HASHBANG: &str = "#!";

#[derive(PartialEq, PartialOrd, Debug)]
enum LicenseCheckRes {
    Missing,
    Found,
    Outdated,
}

pub struct Formatter {
    files: Vec<PathBuf>,
    header: String,
    config: Config,
    silent: bool,
    dry_run: bool,
}

impl Formatter {
    /// Create an instance of the Formatter with the validated config and header that
    /// has template filled with correct values.
    ///
    /// The command line arguments for `licensee` and `license_year` take precedence
    /// over values provided in the config.
    ///
    /// The files to update the license header for are:
    /// - the positional command line arguments if available
    /// - otherwise, all the python files recursively found under the cwd excluding
    ///   `*.egg-info/`, dist/, __pycache__/, and hidden directories and files.
    ///
    /// Args:
    ///     - files: Python files to run license header formatter on. If any provided, no
    ///         search for files is run.
    ///     - cl_licensee: The command line input for the `licensee` to use in header template.
    ///     - cl_license_year: The command line input for the `license_year` to use in the header template.
    ///     - silent: The command line input for whether to print results of checks and formatting.
    ///     - dry_run: The command line input for whether to only run check for correct license header
    ///         without running formatter.
    ///
    /// Returns: A Formatter.
    pub fn new(
        files: &[String],
        cl_licensee: &Option<String>,
        cl_license_year: &Option<u16>,
        silent: bool,
        dry_run: bool,
    ) -> Self {
        let mut config = read_config();
        if config.license_header_template.is_none() {
            println!("No license header found in config file.");
            exit(1);
        }

        // check for a licensee input from the command line
        if let Some(licensee) = cl_licensee {
            config.licensee = Some(licensee.to_string());
        }

        // check for a license_year input from the command line
        if let Some(year) = cl_license_year {
            config.license_year = i64::from(*year);
        }

        // generate the header from the template and command line arguments
        let header = format_header(&config);

        // the files to update the license header for are:
        // - the positional command line arguments if available
        // - otherwise, all the python files recursively found under the cwd excluding
        //         *.egg-info/, dist/, __pycache__/, and hidden directories and files.
        let files: Vec<PathBuf> = if !files.is_empty() {
            files
                .iter()
                .map(PathBuf::from)
                .filter(|path| path.exists() && path.extension().unwrap() == "py")
                .collect()
        } else {
            let mut python_files: Vec<PathBuf> = vec![];
            let ignore_dirs: [Regex; 4] = [
                Regex::new(r"^dist$").unwrap(),
                Regex::new(r"^__pycache__$").unwrap(),
                Regex::new(r"^.*\.egg-info$").unwrap(),
                Regex::new(r"^\..*$").unwrap(),
            ];
            find_python_files(PathBuf::from("./"), &mut python_files, &ignore_dirs);
            python_files
        };

        Formatter {
            files,
            header,
            config,
            silent,
            dry_run,
        }
    }

    /// Run the license header check and formatter on the collected files with multithreading.
    pub fn format_files(&self) {
        // total the number of files that had incorrect license headers.
        let num_to_fix: i32 = self
            .files
            .par_iter()
            .map(|file| self.format_file(file) as i32)
            .sum();

        if !self.silent && self.dry_run {
            println!("\n{} files to fix.", num_to_fix.to_string().red().bold());
        } else if !self.silent && !self.dry_run {
            println!("\n{} files fixed.", num_to_fix.to_string().red().bold());
        }
        // the exit code is the number of files that had incorrect license headers.
        exit(num_to_fix);
    }

    /// Run the checker and formatter on a file. If `dry_run` is True, only runs the checker.
    ///
    /// Args:
    ///     - file: The Path to the file to run the check and formatter on.
    ///
    /// Returns whether the file had a correct header or not.
    fn format_file(&self, file: &PathBuf) -> bool {
        // Open the file.
        let mut f = if self.dry_run {
            File::options().read(true).open(file).unwrap()
        } else {
            File::options().read(true).write(true).open(file).unwrap()
        };
        let file_path = file.as_path().to_str().unwrap();

        // extract the first comment block
        let (found_header, insert_at) = find_first_comment(&f);
        let mut needs_fix = false;

        // run the checker to see if the header is missing, found, or outdated
        // and call appropriate function
        match check_license(&found_header, &self.config) {
            (_, LicenseCheckRes::Missing) => {
                needs_fix = true;
                if !self.silent {
                    println!("{}: License header missing.", file_path.red().bold());
                }
                if !self.dry_run {
                    insert_header(&mut f, &self.header, insert_at);
                }
            }
            (_, LicenseCheckRes::Found) => {
                if !self.silent {
                    println!("{}: License header found.", file_path.cyan().bold());
                }
            }
            (replace, LicenseCheckRes::Outdated) => {
                needs_fix = true;
                if !self.silent {
                    println!(
                        "{} License header outdated.",
                        file_path.bright_yellow().bold()
                    );
                }
                debug!("Found {found_header}");
                debug!("Replace {replace}");
                if !self.dry_run {
                    update_header(&mut f, &replace, &self.header);
                }
            }
        }

        needs_fix
    }
}

/// Check if the found comment block is a valid license header.
///
/// Args:
///     - comment_block: The first comment block in a Python file.
///     - config: The config for the formatter.
///
/// Returns: The result of the check LicenseCheckRes::{Missing, Outdated, Found}.
///
fn check_license(comment_block: &str, config: &Config) -> (String, LicenseCheckRes) {
    // Clean license headers by removing # from the beginning and trimming whitespaces
    let clean_header = |lines: &str| {
        lines
            .lines()
            .map(|line| line.trim_start_matches(COMMENT).trim().to_string())
            // .filter(|line| !line.is_empty())
            .collect::<Vec<String>>()
    };
    // keep track if the year in the license header is outdated.
    let mut outdated = false;
    let mut header_template = config.license_header_template.clone().unwrap();

    // By this point we have made sure that the licensee field of the config
    // is filled if the placeholder {licensee} is found in the template for the header
    if let Some(licensee) = config.licensee.as_ref() {
        header_template = header_template.replace("{licensee}", licensee);
    }

    // clean both the license headers
    let comments = clean_header(comment_block);
    let templates = clean_header(&header_template);

    debug!("Found header {comment_block} expected {header_template}.");

    // If the length of the cleaned headers are different, then the headers are different
    let template_num_lines = templates.len();
    if comments.len() < template_num_lines {
        debug!(
            "The found header's length {} is less than the templates {}.",
            comments.len(),
            template_num_lines,
        );
        return (String::from(""), LicenseCheckRes::Missing);
    }

    let mut template_line_num = 0usize;
    let mut cur_template_line = &templates[template_line_num];
    let mut found_license_start = 0usize;

    for (idx, comment_line) in comments.iter().enumerate() {
        let comment_words = comment_line.split(" ").collect::<Vec<_>>();
        let template_words = cur_template_line.split(" ").collect::<Vec<_>>();

        if comment_words.len() != template_words.len() {
            debug!("Length of line {comment_words:?} not equal to {template_words:?}.");
            // already had found lines of the correct header matching
            // but matching failed for this line
            if template_line_num != 0 {
                return (String::from(""), LicenseCheckRes::Missing);
            }
            continue;
        }
        let mut matched_words = 0usize;
        for (comment_word, template_word) in comment_words.into_iter().zip(&template_words) {
            match template_word.to_string().as_str() {
                "{year}" => {
                    // check if the {year} template placeholder matches with a number in
                    // the comment block
                    if let Ok(year) = comment_word.parse::<i64>() {
                        // if a number, check if the year is the same as the year provided in
                        // in the config. It is outdated if not the same as the license year
                        // from the config.
                        if year != config.license_year {
                            outdated = true;
                        }
                    } else {
                        // if parsing fails, then the headers are different.
                        debug!("Failed to parse year.");
                        // already had found lines of the correct header matching
                        // but matching failed for this line
                        if template_line_num != 0 {
                            return (String::from(""), LicenseCheckRes::Missing);
                        }
                        continue;
                    }
                }
                word => {
                    // if the words are different then the headers are different.
                    if comment_word != word {
                        debug!("Different words comment {comment_word} template {word}.");
                        // already had found lines of the correct header matching
                        // but matching failed for this line
                        if template_line_num != 0 {
                            return (String::from(""), LicenseCheckRes::Missing);
                        }
                        continue;
                    }
                }
            }
            matched_words += 1;
        }
        if template_line_num == 0 {
            // This is the line where we started matching the header correctly
            found_license_start = idx;
        }

        if matched_words == template_words.len() {
            template_line_num += 1;
        }
        if template_line_num != template_num_lines {
            cur_template_line = &templates[template_line_num];
        } else {
            break;
        }
    }

    if template_line_num == template_num_lines {
        if outdated {
            let found_license_end = found_license_start + template_line_num;
            let found_header: String = comment_block.lines().collect::<Vec<&str>>()
                [found_license_start..found_license_end]
                .join("\n");
            return (found_header, LicenseCheckRes::Outdated);
        } else {
            return (String::from(""), LicenseCheckRes::Found);
        }
    }
    (String::from(""), LicenseCheckRes::Missing)
}

/// Find the first comment block for a Python file and the byte index to potentially insert a
/// license header at. Skip hashbangs, and empty lines before the first none empty line.
///
/// Args:
///     - file: The Python file.
///
/// Returns: The first comment block and the position
fn find_first_comment(file: &File) -> (String, usize) {
    // will be used to build the comment block
    let mut found_header: String = String::new();
    // the byte index in the file where a new license header should be inserted
    let mut insert_at: usize = 0;

    for line in io::BufReader::new(file).lines().map_while(|line| line.ok()) {
        // haven't found a comment yet
        if found_header.is_empty() {
            if line.starts_with(HASHBANG) {
                // TODO: Maybe ignore if not the first line of the file
                // skip hash bang
                insert_at += line.len() + 1;
                continue;
            } else if line.trim().is_empty() {
                // line only contains whitespaces
                continue;
            } else if line.starts_with(COMMENT) {
                // the first comment line.
                // don't increment insert_at. If this comment ends up being an
                // incorrect header, the correct header is inserted before it.
                found_header += &line;
                found_header += "\n";
            } else {
                break;
            }
        // the first comment
        } else if line.starts_with(COMMENT) {
            found_header += &line;
            found_header += "\n";
        // first none comment line
        } else {
            break;
        }
    }

    (found_header, insert_at)
}

/// Inserts a license header into a file.
///
/// Args:
///     - file: The Python file in which the license_header is being inserted.
///     - license_header: The license header being inserted.
///     - insert_at: The byte index in the file where the license header will be
///         inserted.
///
fn insert_header(file: &mut File, license_header: &str, insert_at: usize) {
    // The content of the file
    let mut content = String::new();

    // move cursor to begining and read all the content
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_to_string(&mut content)
        .expect("Failed to read file");
    file.set_len(0).unwrap();
    // move cursor to begining again to avoid strange writing
    file.seek(SeekFrom::Start(0)).unwrap();

    if insert_at == 0 {
        // inserting at the beginning involves writing the header then the rest
        // of the content.
        file.write_all(license_header.as_bytes()).unwrap();
        if content.chars().next() == COMMENT.chars().next() {
            // if the first character of the file is a comment,
            // then add a new line before writing the original content.
            file.write_all("\n".as_bytes()).unwrap();
        }
        file.write_all(content.as_bytes()).unwrap();
    } else {
        // inserting elsewhere involves
        //  - splitting the content at the insert_at byte index,
        //  - writing the first half of the content,
        //  - writing the correct header,
        //  - writing the remainder of the header,
        let after_header = content.split_off(insert_at);

        // if the content before the header would have been whitespaces,
        // don't write it.
        if !content.trim().is_empty() {
            file.write_all(content.as_bytes()).unwrap();
        }

        file.write_all(license_header.as_bytes()).unwrap();
        if after_header.chars().next() == COMMENT.chars().next() {
            // if the first character of the remaining content is a comment,
            // then add a new line before writing the original content.
            file.write_all("\n".as_bytes()).unwrap();
        }
        file.write_all(after_header.as_bytes()).unwrap();
    }
}

/// Updates a license header in a file with the correct one.
///
/// Args:
///     - file: The Python file in which the license_header is being inserted.
///     - existing_header: The existing header in the file.
///     - license_header: The license header being inserted.
///
fn update_header(file: &mut File, exisiting_header: &str, license_header: &str) {
    let mut content = String::new();

    // move cursor to begining and read all the content
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_to_string(&mut content)
        .expect("Failed to read file");
    file.set_len(0).unwrap();
    // move cursor to begining again to avoid strange writing
    file.seek(SeekFrom::Start(0)).unwrap();
    if !content.ends_with('\n') {
        debug!("Add new line to end of content");
        content += "\n";
    }

    let split_at_idx = content.find(exisiting_header).unwrap();
    // split the content at the start of the header
    let (before_header, after_header_inclusive) = content.split_at_checked(split_at_idx).unwrap();

    // Write the content before the header, the new header, then the content after the original header.
    file.write_all(before_header.as_bytes()).unwrap();
    debug!("License header: <{license_header}>");
    file.write_all(license_header.as_bytes()).unwrap();

    // remove the existing header from the content
    let after_header = &after_header_inclusive[exisiting_header.len()..];
    debug!("After header: <{after_header}>");
    debug!("After header inclusive: <{after_header_inclusive}>");

    if after_header.chars().next() == COMMENT.chars().next() {
        file.write_all("\n".as_bytes()).unwrap();
    }
    file.write_all(after_header.as_bytes()).unwrap();
}

/// Replace the place holders in the header template with the values from the config.
///
/// Args:
///     - config: Reference to the `Config` with the header template and values to replace
///         placeholders with.
///
/// Returns: The header with template placeholders filled out.
fn format_header(config: &Config) -> String {
    // there has already been a check for a header so unwrap is safe.
    let mut header = config.license_header_template.as_ref().unwrap().clone();

    // replace the {year} and {licensee}
    if header.contains("{licensee}") {
        // exit if {licensee} placeholder is provided but not a value.
        if config.licensee.is_none() {
            println!(
                "{{licensee}} template found in header but no value provided in config or command line."
            );
            exit(1);
        }
        header = header.replace("{licensee}", config.licensee.as_ref().unwrap());
    }

    header = header.replace("{year}", &config.license_year.to_string());

    // Add # to the beginning of each line of the license header if it did not contain one.
    header
        .lines()
        .map(|line| {
            let line = line.trim();
            if !line.starts_with(COMMENT) {
                COMMENT.to_string() + " " + line + "\n"
            } else {
                line.to_string() + "\n"
            }
        })
        .collect::<String>()
}

/// Recursively finds all the python files in a directory ignoring the following dirs:
///     - *.egg-info/, dist/, __pycache__/, and hidden directories and files.
///
/// Args:
///     - cur_dir: The current directory where python files are being searched for.
///     - python_files: Vector being used to accumulate found python files.
///     - ingore_fitd: Array containing the regex for the directories to ignore.
fn find_python_files(cur_dir: PathBuf, python_files: &mut Vec<PathBuf>, ignore_dirs: &[Regex; 4]) {
    match read_dir(cur_dir) {
        // TODO: maybe handle failing to read directory differently?
        Err(_) => {}
        Ok(files) => files
            .filter_map(|entry| entry.ok())
            .for_each(|entry: DirEntry| {
                let path = entry.path();
                let name = entry.file_name().into_string().unwrap();

                // make suree that the path is a directory and not one of the
                // ones to ignore, then recusively check if it has python files.
                if path.is_dir() && !ignore_dirs.iter().any(|re| re.is_match(&name)) {
                    find_python_files(path, python_files, ignore_dirs);
                } else if let Some(ext) = path.extension()
                    && ext == "py"
                {
                    python_files.push(path);
                }
            }),
    }
}

#[test]
fn test_format() {
    use tempfile::NamedTempFile;

    env_logger::init();
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    let mut tmp = tmp.reopen().unwrap();

    let config = Config {
        license_header_template: Some("# {year} {licensee}".to_string()),
        licensee: Some("Acme Corp".to_string()),
        license_year: 2025,
        avoid: vec![],
    };

    let header = format_header(&config);
    assert_eq!("# 2025 Acme Corp\n".to_string(), header);

    let test_formatter = Formatter {
        files: vec![],
        header,
        config: config.clone(),
        silent: true,
        dry_run: false,
    };

    let test_fixtures = [
        (
            // 0
            "",                       // content of the file
            "",                       // expected comment block extracted from file
            0,                        // index at which the header would be inserted
            LicenseCheckRes::Missing, // result of check_license
            true,                     // result of Formatter.format_file()
            "# 2025 Acme Corp\n",     // the expected file content of the formatted file
        ),
        (
            // 1
            "\n\n",
            "",
            0,
            LicenseCheckRes::Missing,
            true,
            "# 2025 Acme Corp\n\n\n",
        ),
        (
            // 2
            "# Comment",
            "# Comment\n",
            0,
            LicenseCheckRes::Missing,
            true,
            "# 2025 Acme Corp\n\n# Comment",
        ),
        (
            // 4
            "\n\n# Comment",
            "# Comment\n",
            0,
            LicenseCheckRes::Missing,
            true,
            "# 2025 Acme Corp\n\n\n# Comment",
        ),
        (
            // 4
            "#!/usr/bin/python\n\n# Comment",
            "# Comment\n",
            18,
            LicenseCheckRes::Missing,
            true,
            "#!/usr/bin/python\n# 2025 Acme Corp\n\n# Comment",
        ),
        (
            // 5
            "#!/usr/bin/python\n\n# 2025 Acme Corp",
            "# 2025 Acme Corp\n",
            18,
            LicenseCheckRes::Found,
            false,
            "#!/usr/bin/python\n\n# 2025 Acme Corp",
        ),
        (
            // 6
            "#!/usr/bin/python\n\n# 2024 Acme Corp",
            "# 2024 Acme Corp\n",
            18,
            LicenseCheckRes::Outdated,
            true,
            "#!/usr/bin/python\n\n# 2025 Acme Corp\n\n",
        ),
        (
            // 7
            "#!/usr/bin/python\n#\n# 2024 Acme Corp\n",
            "#\n# 2024 Acme Corp\n",
            18,
            LicenseCheckRes::Outdated,
            true,
            "#!/usr/bin/python\n#\n# 2025 Acme Corp\n\n",
        ),
        (
            // 8
            "#!/usr/bin/python\n\n# 2024 Acme Corp\n# ",
            "# 2024 Acme Corp\n# \n",
            18,
            LicenseCheckRes::Outdated,
            true,
            "#!/usr/bin/python\n\n# 2025 Acme Corp\n\n# \n",
        ),
        (
            // 9
            "#!/usr/bin/python\n# 2024 Acme Corp\n\n# More Comment",
            "# 2024 Acme Corp\n",
            18,
            LicenseCheckRes::Outdated,
            true,
            "#!/usr/bin/python\n# 2025 Acme Corp\n\n\n# More Comment\n",
        ),
        (
            // 10
            "#!/usr/bin/python\n\n# 2024 Acme Corp\n# Wrong format",
            "# 2024 Acme Corp\n# Wrong format\n",
            18,
            LicenseCheckRes::Outdated,
            true,
            "#!/usr/bin/python\n\n# 2025 Acme Corp\n\n# Wrong format\n",
        ), // insert_header does not add new line to the end of the file
    ];

    for (
        idx,
        (
            content,
            expected_comment,
            expected_insert_at,
            expected_check_res,
            expected_format_res,
            expected_final,
        ),
    ) in test_fixtures.into_iter().enumerate()
    {
        tmp.seek(SeekFrom::Start(0)).unwrap();
        tmp.write_all(content.as_bytes()).unwrap();
        tmp.seek(SeekFrom::Start(0)).unwrap();

        let (first_comment, insert_at) = find_first_comment(&tmp);
        assert_eq!(
            expected_comment, first_comment,
            "[find_first_comment first_comment] Failed for fixture {}.",
            idx
        );
        assert_eq!(
            expected_insert_at, insert_at,
            "[find_first_comment insert at] Failed for fixture {}.",
            idx
        );
        let x: Vec<&str> = first_comment.lines().collect();
        debug!("{x:?}");
        let (_, check_res) = check_license(&first_comment, &config);
        assert_eq!(
            check_res, expected_check_res,
            "[check license] Failed for fixture {}.",
            idx
        );
        let format_res = test_formatter.format_file(&path);
        assert_eq!(
            expected_format_res, format_res,
            "[format_file format_res] Failed for fixture {}.",
            idx
        );

        let mut buffer = String::new();
        tmp.seek(SeekFrom::Start(0)).unwrap();
        tmp.read_to_string(&mut buffer).unwrap();

        assert_eq!(
            expected_final, buffer,
            "[format_file buffer] Failed for fixture {}.",
            idx
        );
        tmp.set_len(0).unwrap();
        tmp.seek(SeekFrom::Start(0)).unwrap();
    }
}
