use crate::{cli::CheckCommand, utils::SWAY_GIT_TAG};
use anyhow::Result;
use forc_pkg::{self as pkg};
use std::path::PathBuf;

pub fn check(command: CheckCommand) -> Result<sway_core::CompileAstResult> {
    let CheckCommand { path, silent_mode } = command;

    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir()?
    };

    pkg::check(&this_dir, silent_mode, SWAY_GIT_TAG)
}
