pub mod defaults;
pub mod parameters;

use anyhow::Result;
use forc_util::{println_red_err, println_yellow_err};
use rustc_version::{version, Version};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The `forc` crate version formatted with the `v` prefix. E.g. "v1.2.3".
///
/// This git tag is used during `Manifest` construction to pin the version of the implicit `std`
/// dependency to the `forc` version.
pub const SWAY_GIT_TAG: &str = concat!("v", clap::crate_version!());

pub(crate) fn forc_cargo_toml_as_str() -> Result<String> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_file = format!("{}/Cargo.toml", cargo_dir);
    let toml_path = Path::new(&cargo_file);

    let mut file = File::open(toml_path)?;
    let mut toml = String::new();
    file.read_to_string(&mut toml)?;

    Ok(toml)
}

pub(crate) fn check_rust_version() -> Result<()> {
    let toml = forc_cargo_toml_as_str()?;
    let rustc_version = match version() {
        Ok(v) => v,
        Err(_e) => {
            println_red_err("rustc was not found in this environment.\n\nPlease see https://www.rust-lang.org/tools/install for more details on how you can install rustc.");
            std::process::exit(0x01);
        }
    };

    let cargo_toml: toml::Value = toml::de::from_str(&toml)?;

    if let Some(table) = cargo_toml.as_table() {
        if let Some(version) = table.get("package").unwrap().get("rust-version") {
            let version_str = &version.as_str().unwrap();
            let forc_rustc_version = Version::parse(version_str)?;
            if rustc_version > forc_rustc_version {
                let warning = format!(
                    "\nFound rustc version {}, which is greater than the suggested version {}\n",
                    &rustc_version, &forc_rustc_version
                );
                println_yellow_err(&warning);
            }
        }
    }

    Ok(())
}
