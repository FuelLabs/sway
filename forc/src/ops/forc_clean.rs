use crate::{cli::CleanCommand, utils::helpers::default_output_directory};
use anyhow::{anyhow, bail, Result};
use std::{path::PathBuf, process};
use sway_utils::{find_manifest_dir, MANIFEST_FILE_NAME};

pub fn clean(command: CleanCommand) -> Result<()> {
    let CleanCommand { path } = command;

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };
    let manifest_dir = match find_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            bail!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            )
        }
    };

    // Clear `<project>/out` directory.
    // Ignore I/O errors telling us `out_dir` isn't there.
    let out_dir = default_output_directory(&manifest_dir);
    let _ = std::fs::remove_dir_all(out_dir);

    // Run `cargo clean`, forwarding stdout and stderr (`cargo clean` doesn't appear to output
    // anything as of writing this).
    process::Command::new("cargo")
        .arg("clean")
        .stderr(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .output()
        .map_err(|e| e)?;

    Ok(())
}
