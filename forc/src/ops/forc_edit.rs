use std::path::PathBuf;

use crate::cli::{AddCommand, RemoveCommand};
use anyhow::{bail, Result};
use forc_pkg::manifest::ManifestFile;

pub fn add(
    /*the command from the user that is typed into the terminal*/ command: AddCommand,
) -> Result<()> {
    // 1. How will forc know where to add a dependency to?

    // type of the variable
    let AddCommand {
        crates: _,     // whatever library the user is trying to add to the toml
        manifest_path, // the path to forc.toml
    } = command; // variable that we got from the add function

    // matches the path to the forc.toml
    let dir = match manifest_path {
        Some(ref path) => PathBuf::from(path), // if manifest_path, then manifest_path
        None => std::env::current_dir()?,      // if not, then current directory
    };
    let manifest = ManifestFile::from_dir(&dir)?;
    let pkg_manifest = if let ManifestFile::Package(pkg_manifest) = &manifest {
        pkg_manifest
    } else {
        bail!("forc-doc does not support workspaces.")
    };
    // 2. How will we find the dependency block in the forc.toml?

    // 3. Write new dependency to that block (name and version)
    Ok(())
}
pub fn remove(_command: RemoveCommand) -> Result<()> {
    Ok(())
}
