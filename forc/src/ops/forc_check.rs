use crate::{cli::CheckCommand, utils::SWAY_GIT_TAG};
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
    let manifest = ManifestFile::from_dir(&this_dir, SWAY_GIT_TAG)?;
    let plan = pkg::BuildPlan::load_from_manifest(&manifest, locked, offline, SWAY_GIT_TAG)?;

    pkg::check(&plan, silent_mode, SWAY_GIT_TAG)
}
