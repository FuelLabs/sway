use crate::cli::CheckCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, ManifestFile};
use std::path::PathBuf;

pub fn check(command: CheckCommand) -> Result<sway_core::CompileAstResult> {
    let CheckCommand {
        path,
        offline_mode: offline,
        silent_mode,
        locked,
    } = command;

    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&this_dir)?;
    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;

    let (_, ast_res) = pkg::check(&plan, silent_mode)?;
    Ok(ast_res)
}
