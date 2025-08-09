use crate::utils::{Config, read_config};
use chrono::{Datelike, Utc};
use colored::Colorize;
use rayon::prelude::*;
use regex::Regex;
use std::fs::{DirEntry, File, read_dir};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::exit;

const COMMENT: &str = "#";
const HASHBANG: &str = "#!";

enum LicenseMatchRes {
    Update,
    Insert,
    Skip,
}

fn match_license(comment_block: &str, config: &Config) -> LicenseMatchRes {
    let clean_header = |lines: &str| {
        lines
            .lines()
            .map(|line| line.trim_start_matches(COMMENT).trim().to_string())
            .filter(|line| !line.is_empty())
            .collect::<Vec<String>>()
    };

    // let clean_header = |lines: &str| {
    //     lines
    //         .lines()
    //         .map(|line| line.trim_start_matches(COMMENT).trim().to_string())
    //         .filter(|line| !line.is_empty())
    //         .collect::<String>()
    // };

    let mut header_template = config.license_header.clone().unwrap();
    // header_template = header_template.replace("{year}", r"(?<year>\d{4})");

    if let Some(licensee) = config.licensee.as_ref() {
        header_template = header_template.replace("{licensee}", licensee);
    }

    let comments = clean_header(comment_block);
    let templates = clean_header(&header_template);
    let mut years: Vec<i64> = vec![];
    // let regex = Regex::new(&templates).unwrap();

    // if regex.is_match(&comments) {
    //     let cur_year = i64::from(Utc::now().year());
    //     if regex
    //         .captures_iter(&comments)
    //         .any(|cap| cap["year"].parse::<i64>().unwrap() != cur_year)
    //     {
    //         return LicenseMatchRes::Update;
    //     } else {
    //         return LicenseMatchRes::Skip;
    //     }
    // } else {
    //     return LicenseMatchRes::Insert;
    // }

    // println!(
    //     "Found header {} expected {}",
    //     comment_block, header_template
    // );

    if comments.len() != templates.len() {
        // println!(
        //     "Different length for headers {} not equal to {}",
        //     comments.len(),
        //     templates.len()
        // );
        return LicenseMatchRes::Insert;
    }

    for (comment_line, template_line) in comments.iter().zip(templates) {
        let comment_words = comment_line.split(" ").collect::<Vec<_>>();
        let template_words = template_line.split(" ").collect::<Vec<_>>();

        if comment_words.len() != template_words.len() {
            // println!(
            //     "Different length for line {:?} not equal to {:?}",
            //     comment_words, template_words,
            // );
            return LicenseMatchRes::Insert;
        }

        for (comment_word, template_word) in comment_words.into_iter().zip(template_words) {
            match template_word {
                "{year}" => {
                    if let Ok(year) = comment_word.parse::<i64>() {
                        // println!("Parsed year {}", year);
                        years.push(year);
                    } else {
                        // println!("Failed to parse year");
                        return LicenseMatchRes::Insert;
                    }
                }
                word => {
                    if comment_word != word {
                        // println!("Different words comment {} template {}", comment_word, word);
                        return LicenseMatchRes::Insert;
                    }
                }
            }
        }
    }

    let cur_year = i64::from(Utc::now().year());
    // println!("Years found {:?}", years);
    if years.iter().any(|year| year != &cur_year) {
        LicenseMatchRes::Update
    } else {
        LicenseMatchRes::Skip
    }
}

fn find_first_comment(file: &File) -> (String, usize) {
    let mut found_header: String = String::new();
    let mut insert_at: usize = 0;

    for line in io::BufReader::new(file).lines().map_while(|line| line.ok()) {
        if found_header.is_empty() {
            // haven't found a comment year
            if line.starts_with(HASHBANG) {
                insert_at += line.len() + 1;
                continue;
            } else if line.trim().is_empty() {
                insert_at += line.len();
                continue;
            } else if line.starts_with(COMMENT) {
                found_header += &line;
                found_header += "\n";
            } else {
                break;
            }
        } else if line.starts_with(COMMENT) {
            found_header += &line;
            found_header += "\n";
        } else {
            break;
        }
    }

    (found_header, insert_at)
}

fn insert_header(file: &mut File, license_header: &str, insert_at: usize) {
    let mut content = String::new();

    // move cursor to begining and read all the content
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_to_string(&mut content)
        .expect("Failed to read file");
    file.set_len(0).unwrap();
    // move cursor to begining again to avoid strange writing
    file.seek(SeekFrom::Start(0)).unwrap();

    if insert_at == 0 {
        file.write_all(license_header.as_bytes()).unwrap();
        if content.chars().next() == COMMENT.chars().next() {
            file.write_all("\n".as_bytes()).unwrap();
        }
        file.write_all(content.as_bytes()).unwrap();
    } else {
        let after_header = content.split_off(insert_at);
        file.write_all(content.as_bytes()).unwrap();
        file.write_all("\n".as_bytes()).unwrap();
        file.write_all(license_header.as_bytes()).unwrap();
        if after_header.chars().next() == COMMENT.chars().next() {
            file.write_all("\n".as_bytes()).unwrap();
        }
        file.write_all(after_header.as_bytes()).unwrap();
    }
}

fn update_header(file: &mut File, exisiting_header: &str, license_header: &str) {
    let mut content = String::new();

    // move cursor to begining and read all the content
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_to_string(&mut content)
        .expect("Failed to read file");
    file.set_len(0).unwrap();
    // move cursor to begining again to avoid strange writing
    file.seek(SeekFrom::Start(0)).unwrap();

    let header_start = content.find(exisiting_header).unwrap();
    let (before_header, after_header_inclusive) = &content.split_at_checked(header_start).unwrap();
    let after_header = &after_header_inclusive[exisiting_header.len()..];

    file.write_all(before_header.as_bytes()).unwrap();
    file.write_all(license_header.as_bytes()).unwrap();
    if after_header.chars().next() == COMMENT.chars().next() {
        file.write_all("\n".as_bytes()).unwrap();
    }
    file.write_all(after_header.as_bytes()).unwrap();
}

fn format_file(
    file: &PathBuf,
    config: &Config,
    license_header: &str,
    silent: bool,
    dry_run: bool,
) -> bool {
    let mut f = File::options().read(true).write(true).open(file).unwrap();
    let file_path = file.as_path().to_str().unwrap();
    let (found_header, insert_at) = find_first_comment(&f);
    let mut needs_fix = false;
    match match_license(&found_header, config) {
        LicenseMatchRes::Insert => {
            needs_fix = true;
            if !silent {
                println!("{}: License header missing.", file_path.red().bold());
            }
            if !dry_run {
                insert_header(&mut f, license_header, insert_at);
            }
        }
        LicenseMatchRes::Skip => {
            if !silent {
                println!("{}: License header found.", file_path.cyan().bold());
            }
        }
        LicenseMatchRes::Update => {
            needs_fix = true;
            if !silent {
                println!(
                    "{} License header outdated.",
                    file_path.bright_yellow().bold()
                );
            }

            if !dry_run {
                update_header(&mut f, &found_header, license_header);
            }
        }
    }

    needs_fix
}

fn format_files(
    files: &Vec<PathBuf>,
    config: &Config,
    header: String,
    silent: bool,
    dry_run: bool,
) {
    let num_to_fix: i32 = files
        .par_iter()
        .map(|file| format_file(file, config, &header, silent, dry_run) as i32)
        .sum();

    if !silent {
        println!("\n{} files to fix.", num_to_fix.to_string().red().bold());
    }
    exit(num_to_fix);
}

/// Replace the place holders in the header with the values from the config
/// and command line arguments.
fn format_header(config: &Config) -> String {
    // replace the {year} and {licensee}
    let mut header = config.license_header.as_ref().unwrap().clone();
    if header.contains("{licensee}") {
        if config.licensee.is_none() {
            println!(
                "{{licensee}} template found in header but no value provided in config or command line."
            );
            exit(1);
        }
        header = header.replace("{licensee}", config.licensee.as_ref().unwrap());
    }

    header = header.replace("{year}", &config.license_year.to_string());

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
        + "\n"
}

/// Recursively finds all the python files in a directory.
fn find_python_files(cur_dir: PathBuf, python_files: &mut Vec<PathBuf>, ingore_dirs: &[Regex; 4]) {
    match read_dir(cur_dir) {
        Err(_) => {}
        Ok(files) => files
            .filter_map(|entry| entry.ok())
            .for_each(|entry: DirEntry| {
                let path = entry.path();
                let name = entry.file_name().into_string().unwrap();

                if path.is_dir() && !ingore_dirs.iter().any(|re| re.is_match(&name)) {
                    find_python_files(path, python_files, ingore_dirs);
                } else if let Some(ext) = path.extension()
                    && ext == "py"
                {
                    python_files.push(path);
                }
            }),
    }
}

/// Run the formatter on the given files
pub fn run_format(
    files: &[String],
    licensee: &Option<String>,
    license_year: &Option<u16>,
    silent: bool,
    dry_run: bool,
) {
    let mut config = read_config();
    if config.license_header.is_none() {
        println!("No license header found in config file.");
        exit(1);
    }

    if let Some(cl_licensee) = licensee {
        config.licensee = Some(cl_licensee.to_string());
    }

    if let Some(cl_year) = license_year {
        config.license_year = i64::from(*cl_year);
    }

    let header = format_header(&config);

    let files: Vec<PathBuf> = if !files.is_empty() {
        files
            .iter()
            .map(PathBuf::from)
            .filter(|path| path.exists() && path.extension().unwrap() == "py")
            .collect()
    } else {
        let mut python_files: Vec<PathBuf> = vec![];
        let ingore_dirs: [Regex; 4] = [
            Regex::new(r"^dist$").unwrap(),
            Regex::new(r"^__pycache__$").unwrap(),
            Regex::new(r"^.*\.egg-info$").unwrap(),
            Regex::new(r"^\..*$").unwrap(),
        ];
        find_python_files(PathBuf::from("./"), &mut python_files, &ingore_dirs);
        python_files
    };

    format_files(&files, &config, header, silent, dry_run);
}
