use crate::cli::CheckCommand;
use anyhow::Result;
use forc_pkg::{self as pkg, PackageManifestFile};
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
    let manifest = PackageManifestFile::from_dir(&this_dir)?;
    let manifest_file = ManifestFile::Package(Box::new(manifest));
    let plan = pkg::BuildPlan::from_lock_and_manifest(&manifest_file, locked, offline)?;

    Ok(pkg::check(&plan, terse_mode)?.flat_map(|(_, tp)| CompileResult::new(tp, vec![], vec![])))
}
