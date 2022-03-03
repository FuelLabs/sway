use std::net::AddrParseError;
use std::path::PathBuf;
use std::{fmt, io};

use sway_core::CompileError;
use sway_utils::constants::MANIFEST_FILE_NAME;

#[derive(Debug)]
pub struct CliError {
    pub message: String,
}

impl CliError {
    pub fn manifest_file_missing(curr_dir: PathBuf) -> Self {
        let message = format!(
            "could not find `{}` in `{}` or any parent directory",
            MANIFEST_FILE_NAME,
            curr_dir.display()
        );
        Self { message }
    }

    pub fn parsing_failed(project_name: &str, errors: Vec<CompileError>) -> Self {
        let message = errors
            .iter()
            .map(|e| e.to_friendly_error_string())
            .collect::<Vec<String>>()
            .join("\n");

        Self {
            message: format!("Parsing {} failed: \n{}", project_name, message),
        }
    }

    pub fn wrong_sway_type(project_name: &str, wanted_type: &str, parse_type: &str) -> Self {
        let message = format!(
            "{} is not a '{}' it is a '{}'",
            project_name, wanted_type, parse_type
        );
        Self { message }
    }

    pub fn fuel_core_not_running(node_url: &str) -> Self {
        let message = format!("could not get a response from node at the URL {}. Start a node with `fuel-core`. See https://github.com/FuelLabs/fuel-core#running for more information", node_url);
        Self { message }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl From<&str> for CliError {
    fn from(s: &str) -> Self {
        CliError {
            message: s.to_string(),
        }
    }
}

impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError { message: s }
    }
}

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        CliError {
            message: e.to_string(),
        }
    }
}

impl From<AddrParseError> for CliError {
    fn from(e: AddrParseError) -> Self {
        CliError {
            message: e.to_string(),
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError {
            message: e.to_string(),
        }
    }
}
