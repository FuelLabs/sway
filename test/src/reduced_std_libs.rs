//! This module contains functions for creating reduced versions of the `std` library.

use anyhow::{bail, Context, Ok, Result};
use core::result::Result::Ok as CoreOk;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) const REDUCED_STD_LIBS_DIR_NAME: &str = "reduced_std_libs";
const REDUCED_LIB_CONFIG_FILE_NAME: &str = "reduced_lib.config";

/// Creates the reduced versions of `std` libraries based on the list of
/// modules defined in [REDUCED_LIB_CONFIG_FILE_NAME] file for each reduced library
/// available in the [REDUCED_STD_LIBS_DIR_NAME].
pub fn create() -> Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let reduced_libs_dir = format!("{manifest_dir}/src/e2e_vm_tests/{REDUCED_STD_LIBS_DIR_NAME}");
    let std_lib_src_dir = format!("{manifest_dir}/../sway-lib-std/src");

    create_reduced_std_libs(&std_lib_src_dir, &reduced_libs_dir)
        .context("Cannot create reduced versions of the Sway Standard Library.")?;

    Ok(())
}

fn create_reduced_std_libs(std_lib_src_dir: &str, reduced_libs_dir: &str) -> Result<()> {
    let std_lib_src_dir = Path::new(std_lib_src_dir);
    let reduced_libs_dir = Path::new(reduced_libs_dir);

    for reduced_lib_dir in get_reduced_libs(reduced_libs_dir)? {
        let reduced_lib_config = reduced_lib_dir.join("reduced_lib.config");

        if !reduced_lib_config.exists() {
            bail!(format!("The config file \"{REDUCED_LIB_CONFIG_FILE_NAME}\" cannot be found for the reduced standard library \"{}\".\nThe config file must be at this location: {}",
                reduced_lib_dir.components().next_back().unwrap().as_os_str().to_string_lossy(),
                reduced_lib_config.as_os_str().to_string_lossy()
            ));
        }

        let modules = get_modules_from_config(&reduced_lib_config)?;
        for module in modules {
            let std_lib_module_path = std_lib_src_dir.join(&module);
            let reduced_lib_module_path = reduced_lib_dir.join("src").join(&module);

            copy_module(&std_lib_module_path, &reduced_lib_module_path)?;
        }
    }

    Ok(())
}

fn get_reduced_libs(reduced_libs_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut reduced_libs = Vec::new();

    let entries = fs::read_dir(reduced_libs_dir)?;
    for entry in entries.flatten() {
        if entry.metadata()?.is_dir() {
            reduced_libs.push(entry.path())
        }
    }

    Ok(reduced_libs)
}

fn get_modules_from_config(config_file: &Path) -> Result<Vec<String>> {
    let config = fs::read_to_string(config_file)?;
    let lines = config
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    Ok(lines)
}

fn copy_module(from: &Path, to: &Path) -> Result<()> {
    let from_metadata = match fs::metadata(from) {
        CoreOk(from_metadata) => from_metadata,
        Err(err) => bail!(
            "Cannot get metadata for module file {from:#?}: {}",
            err.to_string()
        ),
    };
    let to_metadata = fs::metadata(to);

    let should_copy = match to_metadata {
        CoreOk(to_metadata) => {
            let to_modification_time = to_metadata.modified()?;
            let from_modification_time = from_metadata.modified()?;

            from_modification_time > to_modification_time
        }
        Err(_) => true, // Destination file doesn't exist, copy always.
    };

    if should_copy {
        fs::create_dir_all(to.parent().unwrap())?;
        if let Err(err) = fs::copy(from, to) {
            bail!("Cannot copy module {from:#?}: {}", err.to_string())
        };
    }

    Ok(())
}
