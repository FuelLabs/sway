use std::path::PathBuf;

use crate::cli::{AddCommand, RemoveCommand};
use anyhow::Result;

pub fn add(command: AddCommand) -> Result<()> {
    // 1. How will forc know where to add a dependency to?
    let AddCommand {
        crates: _,
        manifest_path,
    } = command;

    let _dir = match manifest_path {
        Some(ref path) => PathBuf::from(path),
        None => std::env::current_dir()?,
    };
    // 2. How will we find the dependency block in the forc.toml?
    //
    // 3. Write new dependency to that block (name and version)
    Ok(())
}
pub fn remove(_command: RemoveCommand) -> Result<()> {
    Ok(())
}
