use chrono::{Datelike, Utc};
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;
use toml::Table;

#[derive(Debug, Clone)]
/// Used for configuring the license check and header check/formatter.
///
/// - avoid: The license types to be flagged by the license check if found.
/// - license_header_template: The template to use for the license header.
/// - license_year: The value of the year to replace the `{year}` field in the template.
/// - licensee: The value of the licensee to replace the `{licensee}` field in the template.
pub struct Config {
    pub avoid: Vec<String>,
    pub license_header_template: Option<String>,
    pub license_year: i64,
    pub licensee: Option<String>,
}

impl Config {
    /// Create a default config where the license year is the current year.
    pub fn default() -> Self {
        Self {
            avoid: vec![],
            license_header_template: None,
            license_year: i64::from(Utc::now().year()),
            licensee: None,
        }
    }
}

/// Read the config toml file from the `pyproject.toml` to extract
/// the licenses to avoid, license header template, license year, and licensee
/// if provided.
///
/// Returns: The config structed with fields filled with the the values from
///     the config file.
pub fn read_config() -> Config {
    const TOML_FILE: &str = "pyproject.toml";

    let mut config = Config::default();
    if !Path::new(TOML_FILE).exists() {
        return config;
    }

    // read the toml file as a string
    let toml_str =
        read_to_string(TOML_FILE).unwrap_or_else(|_| panic!("Failed to read {TOML_FILE} file."));
    let main_table = toml_str.parse::<Table>().unwrap();

    // extract the licensepy field from the toml table
    if let Some(licensepy_config) = main_table.get("tool.licensepy")
        && let Some(table) = licensepy_config.as_table()
    {
        // extract the avoid field
        if let Some(to_avoid) = table.get("avoid").and_then(|v| v.as_array()) {
            let licenses_to_avoid: Vec<String> = to_avoid
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            config.avoid = licenses_to_avoid;
        }
        // extract the licensee field
        if let Some(licensee) = table.get("licensee").and_then(|v| v.as_str()) {
            config.licensee = Some(licensee.to_string());
        }

        // extract the license_year field
        if let Some(year) = table.get("license_year").and_then(|v| v.as_integer()) {
            config.license_year = year;
        }

        // extract the license_header_template field
        if let Some(header) = table
            .get("license_header_template")
            .and_then(|v| v.as_str())
        {
            config.license_header_template = Some(header.to_string());
        }
    }

    config
}

/// Get the Python3 version in the current environment.
///
/// Returns: Array of the major, minor, and patch version.
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
