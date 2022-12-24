use crate::cli::CleanCommand;
use anyhow::{anyhow, bail, Result};
use forc_util::{default_output_directory, find_manifest_file};
use std::path::PathBuf;
use sway_utils::MANIFEST_FILE_NAME;

pub fn clean(command: CleanCommand) -> Result<()> {
    let CleanCommand { path } = command;

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };
    let manifest_path = match find_manifest_file(&this_dir) {
        Some(path) => path,
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
    let manifest_dir = manifest_path
        .parent()
        .expect("manifest file has no parent directory");
    let out_dir = default_output_directory(manifest_dir);
    let _ = std::fs::remove_dir_all(out_dir);

    Ok(())
}
