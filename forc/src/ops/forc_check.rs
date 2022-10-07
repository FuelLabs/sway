use crate::cli::CheckCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, ManifestFile};
use std::path::PathBuf;
use sway_core::CompileResult;

pub fn check(command: CheckCommand) -> Result<CompileResult<sway_core::TyProgram>> {
    let CheckCommand {
        path,
        offline_mode: offline,
        terse_mode,
        locked,
    } = command;

    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest = ManifestFile::from_dir(&this_dir)?;
    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest, locked, offline)?;

    Ok(pkg::check(&plan, terse_mode)?.flat_map(|(_, tp)| CompileResult::new(tp, vec![], vec![])))
}
