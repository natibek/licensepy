use std::fs::{File, read_dir, DirEntry};
use std::io::{self, BufRead};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Command;

struct MetaData {
    name: String,
    license: String,
    requirements: Vec<String>,
}
#[derive(Debug)]
enum DistType<PathBuf> {
    EggDir(PathBuf),
    DistDir(PathBuf),
    Info(PathBuf),
}

fn main() {
    let dist_dirs = get_dist_directories();
    let package_dirs = get_package_directories(dist_dirs);
}

fn get_package_directories(dist_dirs: Vec<String>) -> Vec<DistType<PathBuf>> {
// directory needs to end with .egg-info with PKG_INFO or .dist-info with METADATA
// or the info file
    let mut package_dirs: Vec<DistType<PathBuf>> = Vec::new();
    for dir in dist_dirs {
        let package_dir: Vec<DistType<PathBuf>>= match read_dir(dir) {
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
        };
        package_dirs.extend(package_dir);
    }
    println!("{:?}", package_dirs);
    package_dirs
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
    println!("{:?}", dist_dirs);
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

//     if let Ok(lines) = read_lines("./README.md") {
//         for line in lines.map_while(Result::ok) {
//             println!("{}", line);
//         }
//     }
// }

// fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
// where
//     P: AsRef<Path>,
// {
//     let file = File::open(filename)?;
//     Ok(io::BufReader::new(file).lines())
// }
