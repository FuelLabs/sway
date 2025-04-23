use crate::cli::CheckCommand;
use anyhow::Result;
use forc_pkg as pkg;
use forc_pkg::manifest::GenericManifestFile;
use pkg::manifest::ManifestFile;
use std::{path::PathBuf, sync::Arc};
use sway_core::{language::ty, Engines};
use sway_error::handler::Handler;

pub fn check(
    command: CheckCommand,
    engines: &Engines,
) -> Result<(Option<Arc<ty::TyProgram>>, Handler)> {
    let CheckCommand {
        build_target,
        path,
        offline_mode: offline,
        terse_mode,
        locked,
        disable_tests,
        ipfs_node,
        experimental,
        ..
    } = command;

    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };
    let manifest_file = ManifestFile::from_dir(this_dir)?;
    let member_manifests = manifest_file.member_manifests()?;
    let lock_path = manifest_file.lock_path()?;
    let plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        locked,
        offline,
        &ipfs_node.unwrap_or_default(),
    )?;
    let tests_enabled = !disable_tests;

    let mut v = pkg::check(
        &plan,
        build_target,
        terse_mode,
        None,
        tests_enabled,
        engines,
        None,
        &experimental.experimental,
        &experimental.no_experimental,
        sway_core::DbgGeneration::None,
    )?;
    let (res, handler) = v
        .pop()
        .expect("there is guaranteed to be at least one elem in the vector");
    let res = res.and_then(|programs| programs.typed.ok());
    Ok((res, handler))
}
