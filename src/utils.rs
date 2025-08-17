use chrono::{Datelike, Utc};
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;
use toml::Table;

#[derive(Debug)]
pub struct Config {
    pub avoid: Vec<String>,
    pub license_header: Option<String>,
    pub license_year: i64,
    pub licensee: Option<String>,
}

impl Config {
    pub fn default() -> Self {
        Self {
            avoid: vec![],
            license_header: None,
            license_year: i64::from(Utc::now().year()),
            licensee: None,
        }
    }
}

/// Read the config toml
pub fn read_config() -> Config {
    const TOML_FILE: &str = "pyproject.toml";

    let mut config = Config::default();
    if !Path::new(TOML_FILE).exists() {
        return config;
    }

    let toml_str =
        read_to_string(TOML_FILE).unwrap_or_else(|_| panic!("Failed to read {TOML_FILE} file."));
    let main_table = toml_str.parse::<Table>().unwrap();

    if let Some(licensepy_config) = main_table.get("licensepy")
        && let Some(table) = licensepy_config.as_table()
    {
        if let Some(to_avoid) = table.get("avoid").and_then(|v| v.as_array()) {
            let licenses_to_avoid: Vec<String> = to_avoid
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            config.avoid = licenses_to_avoid;
        }
        if let Some(licensee) = table.get("licensee").and_then(|v| v.as_str()) {
            config.licensee = Some(licensee.to_string());
        }

        if let Some(year) = table.get("license_year").and_then(|v| v.as_integer()) {
            config.license_year = year;
        }

        if let Some(header) = table.get("license_header").and_then(|v| v.as_str()) {
            config.license_header = Some(header.to_string());
        }
    }

    config
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
