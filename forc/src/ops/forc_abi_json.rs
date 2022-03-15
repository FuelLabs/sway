use crate::cli::{BuildCommand, JsonAbiCommand};
use anyhow::Result;
use serde_json::{json, Value};
use std::fs::File;

pub fn build(command: JsonAbiCommand) -> Result<Value> {
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
