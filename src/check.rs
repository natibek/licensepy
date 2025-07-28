use crate::metadata::Metadata;
use colored::Colorize;
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
enum DistType {
    EggDir(PathBuf),
    DistDir(PathBuf),
    Info(PathBuf),
}

impl DistType {
    pub fn get_metadata(
        self,
        python_version: &[i32; 3],
        recursive: bool,
        license_to_avoid: &[String],
    ) -> Metadata {
        match self {
            DistType::EggDir(path) => {
                let metadata_path = path.join("PKG-INFO");
                parse_metadata(metadata_path, python_version, recursive, license_to_avoid)
            }
            DistType::DistDir(path) => {
                let metadata_path = path.join("METADATA");
                parse_metadata(metadata_path, python_version, recursive, license_to_avoid)
            }
            DistType::Info(path) => {
                parse_metadata(path, python_version, recursive, license_to_avoid)
            }
        }
    }
}

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

                if path.is_dir() {
                    if filename.ends_with(".egg-info") {
                        Some(DistType::EggDir(path))
                    } else if filename.ends_with(".dist-info") {
                        Some(DistType::DistDir(path))
                    } else {
                        None
                    }
                } else if filename.ends_with(".egg-info") || filename.ends_with(".dist-info") {
                    Some(DistType::Info(path))
                } else {
                    None
                }
            })
            .collect(),
    }
}

fn get_dist_directories() -> Vec<String> {
    // parse the output of `python3 -m site` to find the dist packages
    let output = Command::new("sh")
        .arg("-c")
        .arg("python3 -m site")
        .output()
        .expect("Error running `python3 -m site`. Make sure `python3` installation is valid");

    let text_output = String::from_utf8(output.stdout).unwrap();
    let dist_dirs: Vec<String> = text_output
        .split(['\n', ',', '\''])
        .filter(|s| s.contains("dist-packages") || s.contains("site-packages"))
        .map(|s| s.trim().to_string())
        .collect();
    dist_dirs
}

// get name
// get the License (some might have License-Expression: )
// Classifier: License :: OSI Approved :: BSD License (could be multiple)
// License: BSD
// Requires-Python: >=3.6
// Requires-Dist: coverage ; extra == 'test'
// Requires-Dist: mypy ; extra == 'test'
fn parse_metadata(
    path: PathBuf,
    python_version: &[i32; 3],
    recursive: bool,
    license_to_avoid: &[String],
) -> Metadata {
    let mut requirements: Vec<String> = Vec::new();
    let mut name: String = String::new();

    let mut license: Vec<String> = Vec::new();
    let mut license_classifier: Vec<String> = Vec::new();

    let clean_line =
        move |line: &str, del: char| line.split(del).last().map(str::trim).unwrap().to_string();

    // if let Ok(lines) = read_lines(path) {
    let file = File::open(path).unwrap();
    for line in io::BufReader::new(file).lines().map_while(Result::ok) {
        if line.starts_with("License: ") || line.starts_with("License-Expression: ") {
            license.push(clean_line(&line, ':'));
        } else if line.starts_with("Name: ") {
            name = clean_line(&line, ':');
        } else if line.starts_with("Classifier: License :: OSI Approved :: ") {
            license_classifier.push(clean_line(&line, ':'));
        } else if line.starts_with("Requires-Dist: ") && recursive {
            if line.contains("extra") {
                continue;
            }
            let req_info = clean_line(&line, ':');
            if !req_info.contains(";") {
                let req = req_info
                    .replace(" ", "")
                    .split(['<', '>', '=', '~', '(', ';', '!'])
                    .next()
                    .map(str::trim)
                    .unwrap()
                    .to_string();
                // println!("Requirement {:?}", req);
                requirements.push(req);
            } else if req_info.contains("; python_version") {
                let py_req = clean_line(&req_info, ';');

                if !meets_python_req(&py_req, python_version) {
                    continue;
                }

                let req = req_info
                    .replace(" ", "")
                    .split(['<', '>', '=', '~', '(', ';', '!'])
                    .next()
                    .map(str::trim)
                    .unwrap()
                    .to_string();
                requirements.push(req);
            }
        }
    }

    if license.len() > license_classifier.len() {
        let bad_license = license.iter().any(|item| license_to_avoid.contains(item));
        Metadata {
            name,
            license,
            requirements,
            bad_license,
        }
    } else if license.is_empty() && license_classifier.is_empty() {
        Metadata {
            name,
            license: vec!["?".to_string()],
            requirements,
            bad_license: false,
        }
    } else {
        let bad_license = license_classifier
            .iter()
            .any(|item| license_to_avoid.contains(item));
        Metadata {
            name,
            license: license_classifier,
            requirements,
            bad_license,
        }
    }
}

fn parse_version(version: &str, python_version: &[i32; 3]) -> [i32; 3] {
    let mut parsed_version: Vec<i32> = version
        .split('.')
        .enumerate()
        .map(|(index, s)| s.parse::<i32>().unwrap_or(python_version[index])) // Convert to integer, default 0 if missing
        .collect();

    let mut diff = 3 - parsed_version.len();

    while diff > 0 {
        parsed_version.push(python_version[3 - diff]);
        diff -= 1;
    }

    parsed_version.try_into().unwrap()
}

fn meets_python_req(constraint: &str, python_version: &[i32; 3]) -> bool {
    let cleaned_constraint = constraint
        .replace(' ', "")
        .replace("\'", "")
        .replace("\"", "");

    // println!("Cleaned Constraint {:?} Original {:?}", cleaned_constraint, constraint);

    let re = Regex::new(r#"(==|<=|>=|!=|<|>)(\d+\.\d+(?:\.\d+)?)"#).unwrap();
    if let Some(caps) = re.captures(&cleaned_constraint) {
        let operator = &caps[1];
        let version_str = &caps[2];

        let constraint_version = parse_version(version_str, python_version);
        // println!("Operator {:?} | Version_string {:?} | new Version {:?}", operator, version_str, constraint_version);

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
    let python_version: [i32; 3] = get_python_version();
    let str_version = python_version
        .iter()
        .map(|n| format!("{}", n))
        .collect::<Vec<_>>()
        .join(".");

    let dist_dirs = get_dist_directories();

    if !silent {
        println!("Avoid {:?}", license_to_avoid);
        println!("PYTHON VERSION {:}", str_version);
        println!("Dependencies stored at {:#?}.", dist_dirs);
        println!();
    }

    let package_dist: Vec<DistType> = dist_dirs
        .par_iter()
        .cloned()
        .flat_map(get_package_dir)
        .collect();
    // println!("{:?}", package_dist);

    let dependencies: Vec<Metadata> = package_dist
        .par_iter()
        .cloned()
        .map(|dist| dist.get_metadata(&python_version, recursive, &license_to_avoid))
        .collect();

    let num_dep = dependencies.len();
    // println!("{:?}", dependencies);
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
