use anyhow::Error;
use std::path::PathBuf;

use sway_core::CompileError;
use sway_utils::constants::MANIFEST_FILE_NAME;

pub fn manifest_file_missing(curr_dir: PathBuf) -> anyhow::Error {
    let message = format!(
        "could not find `{}` in `{}` or any parent directory",
        MANIFEST_FILE_NAME,
        curr_dir.display()
    );
    Error::msg(message)
}

pub fn parsing_failed(project_name: &str, errors: Vec<CompileError>) -> anyhow::Error {
    let error = errors
        .iter()
        .map(|e| e.to_friendly_error_string())
        .collect::<Vec<String>>()
        .join("\n");
    let message = format!("Parsing {} failed: \n{}", project_name, error);
    Error::msg(message)
}

pub fn wrong_sway_type(project_name: &str, wanted_type: &str, parse_type: &str) -> anyhow::Error {
    let message = format!(
        "{} is not a '{}' it is a '{}'",
        project_name, wanted_type, parse_type
    );
    Error::msg(message)
}

pub fn fuel_core_not_running(node_url: &str) -> anyhow::Error {
    let message = format!("could not get a response from node at the URL {}. Start a node with `fuel-core`. See https://github.com/FuelLabs/fuel-core#running for more information", node_url);
    Error::msg(message)
}
