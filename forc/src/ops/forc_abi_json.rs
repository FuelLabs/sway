use crate::cli::{BuildCommand, JsonAbiCommand};
use anyhow::Result;
use forc_pkg::ManifestFile;
use serde_json::{json, Value};
use std::fs::File;
use std::path::PathBuf;
use sway_core::TreeType;
use tracing::info;

pub fn build(command: JsonAbiCommand) -> Result<Value> {
    let curr_dir = if let Some(ref path) = command.path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&curr_dir)?;
    manifest.check_program_type(vec![TreeType::Contract, TreeType::Script])?;

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
        let file = File::create(outfile)?;
        let res = if command.minify {
            serde_json::to_writer(&file, &json_abi)
        } else {
            serde_json::to_writer_pretty(&file, &json_abi)
        };
        res?;
    } else if command.minify {
        info!("{json_abi}");
    } else {
        info!("{:#}", json_abi);
    }

    Ok(json_abi)
}
