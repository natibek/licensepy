use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;
use toml::Table;

pub fn read_toml() -> Vec<String> {
    const TOML_FILE: &str = "pyproject.toml";

    if !Path::new(TOML_FILE).exists() {
        return Vec::new();
    }

    let toml_str = read_to_string(TOML_FILE).expect(&format!("Failed to read {} file.", TOML_FILE));
    let main_table = toml_str.parse::<Table>().unwrap();

    if let Some(licensepy_config) = main_table.get("licensepy") {
        if let Some(table) = licensepy_config.as_table() {
            if let Some(to_avoid) = table.get("avoid").and_then(|v| v.as_array()) {
                let vec_of_strings: Vec<String> = to_avoid
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                return vec_of_strings;
            }
        }
    }
    Vec::new()
}

pub fn get_python_version() -> [i32; 3] {
    let output = Command::new("sh")
        .arg("-c")
        .arg("python3 --version")
        .output()
        .expect("Error running `python3 --version`. Make sure `python3` installation is valid");

    let text_output = String::from_utf8(output.stdout).unwrap();
    let version_str = text_output.split(" ").last().unwrap().trim();

    version_str
        .split('.')
        .map(|s| s.parse::<i32>().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
