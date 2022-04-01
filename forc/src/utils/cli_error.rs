use anyhow::{Error, Result};
use std::path::PathBuf;

use forc_pkg::Manifest;
use forc_util::find_manifest_dir;
use sway_core::{parse, CompileError};
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

pub fn wrong_project_type(
    project_name: &str,
    wanted_type: &str,
    parse_type: &str,
) -> anyhow::Error {
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

/// Given the current directory and file type, determines whether the correct file type is present.
pub fn check_tree_type(curr_dir: PathBuf, wanted_type: &str) -> Result<Manifest> {
    match find_manifest_dir(&curr_dir) {
        Some(manifest_dir) => {
            let manifest = Manifest::from_dir(&manifest_dir)?;
            let project_name = &manifest.project.name;
            let entry_string = manifest.entry_string(&manifest_dir)?;

            let parsed_result = parse(entry_string, None);
            match parsed_result.value {
                Some(parse_tree) => {
                    if parse_tree.tree_type.to_string() != wanted_type {
                        Err(wrong_project_type(
                            project_name,
                            wanted_type,
                            &parse_tree.tree_type.to_string(),
                        ))
                    } else {
                        Ok(manifest)
                    }
                }
                None => Err(parsing_failed(project_name, parsed_result.errors)),
            }
        }
        None => Err(manifest_file_missing(curr_dir)),
    }
}
