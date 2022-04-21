pub mod defaults;
pub mod parameters;

use anyhow::{anyhow, Result};
use forc_util::println_yellow_err;
use rustc_version::{version, Version};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The `forc` crate version formatted with the `v` prefix. E.g. "v1.2.3".
///
/// This git tag is used during `Manifest` construction to pin the version of the implicit `std`
/// dependency to the `forc` version.
pub const SWAY_GIT_TAG: &str = concat!("v", clap::crate_version!());

pub(crate) fn forc_rustc_version() -> Result<Version> {
    let local_rustc_version = version().map_err(|e| anyhow!("Failed to locate rustc: {}", e))?;

    let path = std::env::current_dir()?;
    let forc_toml_file = path.join("Forc.toml");
    let toml_path = Path::new(&forc_toml_file);

    let mut file = File::open(toml_path)?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;

    let cargo_toml: toml::Value = toml::de::from_str(&toml)?;

    if let Some(table) = cargo_toml.as_table() {
        if let Some(project) = table.get("project") {
            if let Some(v) = project.get("rust-version") {
                let vs = &v.as_str().unwrap();
                let forc_rustc_version = Version::parse(vs)?;
                if local_rustc_version > forc_rustc_version {
                    let warning = format!(
                        "\nFound rustc version {}. Recommended version is {}\n",
                        &local_rustc_version, &forc_rustc_version
                    );
                    println_yellow_err(&warning);
                }
            }
        }
    }

    Ok(local_rustc_version)
}
