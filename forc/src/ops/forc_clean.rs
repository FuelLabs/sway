use crate::cli::CleanCommand;
use anyhow::{anyhow, bail, Result};
use forc_pkg::manifest::ManifestFile;
use forc_util::default_output_directory;
use std::path::PathBuf;
use sway_utils::{find_parent_manifest_dir, MANIFEST_FILE_NAME};

pub fn clean(command: CleanCommand) -> Result<()> {
    let CleanCommand { path } = command;

    // find manifest directory, even if in subdirectory
    let this_dir = if let Some(ref path) = path {
        PathBuf::from(path)
    } else {
        std::env::current_dir().map_err(|e| anyhow!("{:?}", e))?
    };

    let manifest_dir = match find_parent_manifest_dir(&this_dir) {
        Some(dir) => dir,
        None => {
            bail!(
                "could not find `{}` in `{}` or any parent directory",
                MANIFEST_FILE_NAME,
                this_dir.display(),
            )
        }
    };
    let manifest = ManifestFile::from_dir(&manifest_dir)?;
    // If this is a workspace collect all member paths and clean each of them.
    let paths: Vec<PathBuf> = match manifest {
        ManifestFile::Package(_) => std::iter::once(this_dir).collect(),
        ManifestFile::Workspace(workspace) => workspace.member_paths()?.collect(),
    };

    for member_path in paths {
        // Clear `<project>/out` directory.
        // Ignore I/O errors telling us `out_dir` isn't there.
        let out_dir = default_output_directory(&member_path);
        let _ = std::fs::remove_dir_all(out_dir);
    }

    Ok(())
}
