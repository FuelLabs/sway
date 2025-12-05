//! Utility items shared between forc crates.

use std::path::{Path, PathBuf};

use sway_utils::constants;

pub mod fs_locking;
#[cfg(feature = "tx")]
pub mod tx_utils;

pub const DEFAULT_OUTPUT_DIRECTORY: &str = "out";

pub fn default_output_directory(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(DEFAULT_OUTPUT_DIRECTORY)
}

/// Returns the user's `.forc` directory, `$HOME/.forc` by default.
pub fn user_forc_directory() -> PathBuf {
    dirs::home_dir()
        .expect("unable to find the user home directory")
        .join(constants::USER_FORC_DIRECTORY)
}
