use crate::cli::CheckCommand;
use anyhow::Result;
use forc_pkg::{self as pkg};
use pkg::manifest::ManifestFile;
use std::path::PathBuf;
use sway_core::{language::ty, CompileResult};

pub fn check(command: CheckCommand) -> Result<CompileResult<ty::TyProgram>> {
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
    let manifest_file = ManifestFile::from_dir(&this_dir)?;
    let member_manifests = manifest_file.member_manifests()?;
    let lock_path = manifest_file.lock_path()?;
    let plan =
        pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, locked, offline)?;

    let mut v = pkg::check(&plan, terse_mode)?;
    let res = v
        .pop()
        .expect("there is guaranteed to be at least one elem in the vector")
        .flat_map(|(_, tp)| CompileResult::new(tp, vec![], vec![]));
    Ok(res)
}
