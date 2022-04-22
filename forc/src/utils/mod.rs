pub mod defaults;
pub mod parameters;

use anyhow::{anyhow, Result};
use rustc_version::{version, Version};

/// The `forc` crate version formatted with the `v` prefix. E.g. "v1.2.3".
///
/// This git tag is used during `Manifest` construction to pin the version of the implicit `std`
/// dependency to the `forc` version.
pub const SWAY_GIT_TAG: &str = concat!("v", clap::crate_version!());

pub(crate) fn forc_rustc_version() -> Result<Version> {
    let version = version().map_err(|e| anyhow!("Failed to locate rustc: {}", e))?;
    Ok(version)
}
