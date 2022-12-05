use crate::cli::{AddCommand, RemoveCommand};
use anyhow::Result;

pub fn add(command: AddCommand) -> Result<()> {
    // 1. How will forc know where to add a dependency to?
    //
    // 2. How will we find the dependency block in the forc.toml?
    //
    // 3. Write new dependency to that block (name and version)
    Ok(())
}
pub fn remove(command: RemoveCommand) -> Result<()> {
    Ok(())
}
