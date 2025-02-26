use std::fs::{File, read_dir, DirEntry};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default, Debug)]
struct Metadata {
    name: String,
    license: Vec<String>,
    requirements: Vec<String>,
}

#[derive(Debug)]
enum DistType {
    EggDir(PathBuf),
    DistDir(PathBuf),
    Info(PathBuf),
}

impl DistType {
    fn get_metadata(self) -> Metadata {
        match self {
            DistType::EggDir(path) => {
                let metadata_path = path.join("PKG-INFO");
                parse_metadata(metadata_path)
            },
            DistType::DistDir(path) => {
                let metadata_path = path.join("METADATA");               
                parse_metadata(metadata_path)
            },
            DistType::Info(path) => parse_metadata(path)
        }
    }
}

fn main() {
    let dist_dirs = get_dist_directories();
    println!("{:?}", dist_dirs);
    let package_dist: Vec<DistType> = dist_dirs
                    .into_iter()
                    .flat_map(get_package_dir)
                    .collect();
    println!("{:?}", package_dist);
    for dist in package_dist{
        println!("{:?}", dist);
        let metadata = dist.get_metadata();
        println!("{:?}", metadata);
        println!();
    }
}

fn get_package_dir(dist_dir: String) -> Vec<DistType> {
// directory needs to end with .egg-info with PKG-INFO or .dist-info with METADATA
// or the info file
        match read_dir(dist_dir) {
            Err(why) => panic!("Failed to read directory {}", why),
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
                        .collect()
        }
}


fn get_dist_directories() -> Vec<String> {
// parse the output of `python3 -m site` to find the dist packages
    let output = Command::new("sh")
        .arg("-c")
        .arg("python3 -m site")
        .output()
        .expect("Error running `python3 -m site`. Make sure `python3` is valid");

    let text_output = String::from_utf8(output.stdout).unwrap();
    let dist_dirs: Vec<String> = text_output.split(|c: char| c == '\n' || c == ',' || c == '\'')
                                            .filter(|s| s.contains("dist-packages"))
                                            .map(|s| s.trim().to_string())
                                            .collect();
    dist_dirs
}

// get name
// get the License
// Classifier: License :: OSI Approved :: BSD License (could be multiple)
// License: BSD
// Requires-Python: >=3.6
// Requires-Dist: coverage ; extra == 'test'
// Requires-Dist: mypy ; extra == 'test'
// Requires-Dist: pexpect ; extra == 'test'
// Requires-Dist: ruff ; extra == 'test'
// Requires-Dist: wheel ; extra == 'test'
fn parse_metadata(path: PathBuf) -> Metadata {
    let mut requirements: Vec<String> = Vec::new();
    let mut name: String = String::new();

    let mut license: Vec<String> = Vec::new();
    let mut license_classifier: Vec<String> = Vec::new();
    if let Ok(lines) = read_lines(path) {
        for line in lines.map_while(Result::ok) {
            if line.starts_with("License: ") {
                license.push(line);
            } else if line.starts_with("Name: ") {
                name = line;
            } else if line.starts_with("Classifier: License :: OSI Approved :: ") {
                license_classifier.push(line);
            } else if line.starts_with("Requires-Dist: ") {
                requirements.push(line);
            }
        }
    }
    if license.len() > license_classifier.len() {
        Metadata {name, license, requirements}
    } else {
        Metadata {name, license: license_classifier, requirements}
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
