use crate::cli::{BuildCommand, JsonAbiCommand};
use crate::utils::cli_error::*;
use anyhow::Result;
use forc_pkg::Manifest;
use forc_util::find_manifest_dir;
use serde_json::{json, Value};
use std::fs::File;
use std::path::PathBuf;
use sway_core::{parse, TreeType};
use sway_utils::constants::*;

pub fn build(command: JsonAbiCommand) -> Result<Value> {
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    // Parse manifest and return error if non-contract
    // Note that this same code is in `forc_deploy` & `forc_run` and may be more useful as a helper function.
    // Its only difference is in what functionality it performs once it finds a `Contract`.
    match find_manifest_dir(&curr_dir) {
        Some(manifest_dir) => {
            let manifest = Manifest::from_dir(&manifest_dir)?;
            let project_name = &manifest.project.name;
            let entry_string = manifest.entry_string(&manifest_dir)?;

            let parsed_result = parse(entry_string, None);
            match parsed_result.value {
                Some(parse_tree) => match parse_tree.tree_type {
                    TreeType::Contract => {
                        let build_command = BuildCommand {
                            path: command.path,
                            offline_mode: command.offline_mode,
                            silent_mode: command.silent_mode,
                            minify_json_abi: command.minify,
                            ..Default::default()
                        };

                        let compiled = crate::ops::forc_build::build(build_command)?;
                        let json_abi = json!(compiled.json_abi);

                        if let Some(outfile) = command.json_outfile {
                            let file = File::create(outfile).map_err(|e| e)?;
                            let res = if command.minify {
                                serde_json::to_writer(&file, &json_abi)
                            } else {
                                serde_json::to_writer_pretty(&file, &json_abi)
                            };
                            res.map_err(|e| e)?;
                        } else if command.minify {
                            println!("{}", json_abi);
                        } else {
                            println!("{:#}", json_abi);
                        }

                        Ok(json_abi)
                    }
                    TreeType::Script => {
                        Err(wrong_sway_type(project_name, SWAY_CONTRACT, SWAY_SCRIPT))
                    }
                    TreeType::Predicate => {
                        Err(wrong_sway_type(project_name, SWAY_CONTRACT, SWAY_PREDICATE))
                    }
                    TreeType::Library { .. } => {
                        Err(wrong_sway_type(project_name, SWAY_CONTRACT, SWAY_LIBRARY))
                    }
                },
                None => Err(parsing_failed(project_name, parsed_result.errors)),
            }
        }
        None => Err(manifest_file_missing(curr_dir)),
    }
}
