use crate::metadata::Metadata;
use colored::Colorize;
use log::debug;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::fs::{DirEntry, read_dir};
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;

use crate::print_output::{print_by_license, print_by_package};
use crate::utils::{Config, get_python_version, read_config};

#[derive(Debug, Clone)]
/// Enum to store the distribution type where the package is stored.
enum DistType {
    EggDir(PathBuf),
    DistDir(PathBuf),
    Info(PathBuf),
}

impl DistType {
    /// Gets the metadata for the different types of ways a package can be stored
    /// in an environment.
    ///
    /// Args:
    ///     - python_version: The python version in the cwd.
    ///     - recursive: Whether to get the metadata for the dependencies required by
    ///         of the current being parsed as well.
    ///     - licenses_to_avoid: Array of licenses to avoid.
    ///
    /// Returns: The Metadata for the package.
    pub fn get_metadata(
        self,
        python_version: &[i32; 3],
        recursive: bool,
        licenses_to_avoid: &[String],
    ) -> Metadata {
        match self {
            DistType::EggDir(path) => {
                let metadata_path = path.join("PKG-INFO");
                parse_metadata(metadata_path, python_version, recursive, licenses_to_avoid)
            }
            DistType::DistDir(path) => {
                let metadata_path = path.join("METADATA");
                parse_metadata(metadata_path, python_version, recursive, licenses_to_avoid)
            }
            DistType::Info(path) => {
                parse_metadata(path, python_version, recursive, licenses_to_avoid)
            }
        }
    }
}

/// For a distribution directory, finds all the folders/files containing the package
/// metadata.
///     - For packages stored in `*.egg-info` directories, the metadata will be found
///         in a PKG-INFO file within the directory.
///     - For packages stored in `*.dist-info` directories, the metadata will be found
///         in a METADATA file within the directory.
///     - Other packages will have their metadata in a file name ending with
///         .egg-info or .dist-info
///
/// Args:
///     - dist_dir: The directory to search for packages' info in.
///
/// Return: All the collected DistTypes from the packages found in the dist_dir.
///
fn get_package_dir(dist_dir: String) -> Vec<DistType> {
    // directory needs to end with .egg-info with PKG-INFO or .dist-info with METADATA
    // or the info file
    match read_dir(dist_dir) {
        Err(_) => Vec::new(),
        Ok(files) => files
            .filter_map(|entry| entry.ok())
            .filter_map(|entry: DirEntry| {
                let path = entry.path();
                let filename = entry.file_name().into_string().ok()?;
                match filename {
                    f if f.ends_with(".egg-info") => {
                        if path.is_dir() {
                            Some(DistType::EggDir(path))
                        } else {
                            Some(DistType::Info(path))
                        }
                    }
                    f if f.ends_with(".dist-info") => {
                        if path.is_dir() {
                            Some(DistType::DistDir(path))
                        } else {
                            Some(DistType::Info(path))
                        }
                    }
                    _ => None,
                }
            })
            .collect(),
    }
}

/// Get the directories in which distributions can be stored.
///
/// Returns: List of the strings (paths) where distributions can be found.
pub fn get_dist_directories() -> Vec<String> {
    // parse the output of `python3 -m site` to find the dist packages
    let output = Command::new("sh")
        .arg("-c")
        .arg("python3 -m site")
        .output()
        .expect("Error running `python3 -m site`. Make sure `python3` installation is valid");

    // Sample output:
    // sys.path = [
    //     '/home/nbhm/licensepy',
    //     '/home/nbhm/miniconda3/lib/python313.zip',
    //     '/home/nbhm/miniconda3/lib/python3.13',
    //     '/home/nbhm/miniconda3/lib/python3.13/lib-dynload',
    //     '/home/nbhm/miniconda3/lib/python3.13/site-packages',
    // ]
    // USER_BASE: '/home/nbhm/.local' (exists)
    // USER_SITE: '/home/nbhm/.local/lib/python3.13/site-packages' (doesn't exist)
    // ENABLE_USER_SITE: True

    let text_output = String::from_utf8(output.stdout).unwrap();
    let dist_dirs: Vec<String> = text_output
        .split(['\n', ',', ':'])
        .filter(|s| {
            (s.contains("dist-packages") || s.contains("site-packages"))
                && !s.contains("doesn't exist") // avoids the USER_SITE OR USER_BASE that doesn't exist
        })
        .map(|s| {
            let mut dir = s.split('\'');
            dir.next();
            dir.next().unwrap().trim().to_string()
        })
        .collect();
    dist_dirs
}

/// Parse metadata file for a package.
///
/// Args:
///     - path: Path to the metadata file.
///     - python_version: Version of Python3 in the cwd.
///     - recursive: Whether to get the metadata for the dependencies required by
///         of the current being parsed as well.
///     - licenses_to_avoid: Array of licenses to avoid.
///
/// Returns: Metadata struct with the field filled with extracted information.
fn parse_metadata(
    path: PathBuf,
    python_version: &[i32; 3],
    recursive: bool,
    license_to_avoid: &[String],
) -> Metadata {
    // requirements for the package
    let mut requirements: Vec<String> = Vec::new();
    let mut name: String = String::new();

    let mut license: Vec<String> = Vec::new();
    // closure for cleaning lines from metadata file.
    // Splits by delimiter and returns the last or first element trimmed as an String.
    let clean_line = move |line: &str, del: &[char], first: bool| {
        let mut parts = line.split(del);
        let part = if first {
            parts.next()
        } else {
            parts.next_back()
        };
        part.map(str::trim).unwrap().to_string()
    };

    let file = File::open(path).unwrap();
    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        if line.starts_with("License-Expression: ")
            || line.starts_with("Classifier: License :: OSI Approved :: ")
        {
            // handling cases like => License: BSD and License-Expression: BSD or
            // handling cases like => Classifier: License :: OSI Approved :: BSD License
            // could be multiple
            license.push(clean_line(&line, &[':'], false));
        } else if line.starts_with("Name: ") {
            // handling cases like => Name: numpy
            name = clean_line(&line, &[':'], false);
        } else if line.starts_with("Requires-Dist: ") && recursive {
            // handling cases like => Requires-Dist: coverage ; extra == 'test'
            // ignore if not recursively handling
            if line.contains("extra") {
                // ignore extra requirement
                continue;
            }
            let req_info = clean_line(&line, &[':'], false);
            if !req_info.contains(";") {
                // extracts the name of the requirement.
                let req = clean_line(&req_info, &['<', '>', '=', '~', '(', ';', '!'], true);

                debug!("Requirement {req:?}.");
                requirements.push(req);
            } else if req_info.contains("; python_version") {
                // if there is a python version stated for the requirement, check that it
                // is met by the python version in the cwd.
                let py_req = clean_line(&req_info, &[';'], false);

                if !meets_python_req(&py_req, python_version) {
                    continue;
                }
                let req = clean_line(&req_info, &['<', '>', '=', '~', '(', ';', '!'], true);
                requirements.push(req);
            }
        }
    }

    if license.is_empty() {
        Metadata {
            name,
            license: vec!["?".to_string()],
            requirements,
            bad_license: false,
        }
    } else {
        // choose either the license or license_classifier
        license.sort();
        license.dedup();
        let bad_license = license.iter().any(|item| license_to_avoid.contains(item));
        Metadata {
            name,
            license,
            requirements,
            bad_license,
        }
    }
}

/// Extract Python version from the string used to denote version restriction in metadata
/// (ie for "...>=3.9" the string "3.9" is provided to the function and returns [3.9.0]
/// if 0 is the patch version provided in the `python_version`). If the minor and/or patch
/// version are not found in the string, they are replaced by the respective versions
/// from the python_version.
///
/// Args:
///     - version: The version string found in metadata.
///     - python_version: The version of Python in the cwd.
///
/// Returns: An array of the major, minor, patch version extracted from the version string.
///
fn parse_version(version: &str, python_version: &[i32; 3]) -> [i32; 3] {
    let mut parsed_version: Vec<i32> = version
        .split('.')
        .enumerate()
        .map(|(index, s)| s.parse::<i32>().unwrap_or(python_version[index]))
        .collect();

    let mut diff = 3 - parsed_version.len();

    // if the any of the version numbers are missing, replace with the respective
    // version number from the python_version
    while diff > 0 {
        parsed_version.push(python_version[3 - diff]);
        diff -= 1;
    }

    parsed_version.try_into().unwrap()
}

/// Check if a provided constraint for a package is met by a python_version.
///
/// Args:
///     - constraint: the constraint for a package.
///     - python_version: the Python3 version in the cwd.
///
/// Returns: Whether the version constraint was met.
fn meets_python_req(constraint: &str, python_version: &[i32; 3]) -> bool {
    let cleaned_constraint = constraint
        .replace(' ', "")
        .replace("\'", "")
        .replace("\"", "");

    let re = Regex::new(r#"(==|<=|>=|!=|<|>)(\d+\.\d+(?:\.\d+)?)"#).unwrap();
    if let Some(caps) = re.captures(&cleaned_constraint) {
        // use regex to extract the operator and version string.
        let operator = &caps[1];
        let version_str = &caps[2];

        let constraint_version = parse_version(version_str, python_version);
        debug!(
            "Operator {operator:?} | Version_string {version_str:?} | new Version {constraint_version:?}."
        );

        match operator {
            "<=" => *python_version <= constraint_version,
            ">=" => *python_version >= constraint_version,
            "<" => *python_version < constraint_version,
            ">" => *python_version > constraint_version,
            "==" => *python_version == constraint_version,
            "!=" => *python_version != constraint_version,
            _ => false,
        }
    } else {
        false
    }
}

/// Run the license checker by extracting all the package info including licenses.
///
/// Args:
///     - recursive: Whether to get the metadata for the dependencies required by
///         of the current being parsed as well.
///     - by_package: Whether to group packages by package or license when printing.
///     - ignore_toml: Whether to ignore the config file.
///     - silent: Whether to print results of checks.
///     - fail_print: Whether to only print the failures (when a license flagged to avoid
///         is found).
pub fn run_check(
    recursive: bool,
    by_package: bool,
    ignore_toml: bool,
    silent: bool,
    fail_print: bool,
) {
    let config: Config = if ignore_toml {
        Config::default()
    } else {
        read_config()
    };
    let license_to_avoid: Vec<String> = config.avoid;
    // get the python version in the cwd
    let python_version: [i32; 3] = get_python_version();
    let str_version = python_version
        .iter()
        .map(|n| format!("{n}"))
        .collect::<Vec<_>>()
        .join(".");

    // get the distribution directories
    let dist_dirs = get_dist_directories();

    if !silent {
        println!("Avoid {license_to_avoid:?}");
        println!("PYTHON VERSION {str_version:}");
        println!("Dependencies stored at {dist_dirs:#?}.");
        println!();
    }

    let package_dist: Vec<DistType> = dist_dirs
        .par_iter()
        .cloned()
        .flat_map(get_package_dir)
        .collect();
    debug!("{package_dist:?}.");

    // multithreaded extraction of metadata
    let dependencies: Vec<Metadata> = package_dist
        .par_iter()
        .cloned()
        .map(|dist| dist.get_metadata(&python_version, recursive, &license_to_avoid))
        .collect();
    debug!("{dependencies:?}.");

    let num_dep = dependencies.len();
    let num_bad_license: i32 = dependencies
        .iter()
        .filter(|dep| dep.bad_license)
        .count()
        .try_into()
        .unwrap();

    if !silent {
        if by_package {
            print_by_package(dependencies, recursive, fail_print);
        } else {
            print_by_license(dependencies, &license_to_avoid, recursive, fail_print);
        }
        println!();
        println!("Found {} total dependencies.", num_dep.to_string().cyan());
        println!(
            "Found {} dependencies with licenses to avoid.",
            num_bad_license.to_string().cyan()
        );
    }

    exit(num_bad_license);
}
