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
    // Check if the manifest refers to a single package
    let plan = if let ManifestFile::Package(package_manifest) = manifest_file {
        // Check if the package resides in a workspace, if that is the case only check package
        // itself and its dependencies.
        if package_manifest.workspace()?.is_some() {
            plan.member_plan(&package_manifest)?
        } else {
            // If this is indeed a single package we do not need to get the member_plan from the
            // BuildPlan as the dependency graph would not change.
            plan
        }
    } else {
        // If the manifest file refers to a workspace we are going to be checking every member of
        // the workspace.
        plan
    };

    Ok(pkg::check(&plan, terse_mode)?.flat_map(|(_, tp)| CompileResult::new(tp, vec![], vec![])))
}
