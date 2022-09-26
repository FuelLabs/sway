use crate::ops::forc_doc;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Command {
    /// Path to the Forc.toml file. By default, Cargo searches for the Forc.toml
    /// file in the current directory or any parent directory.
    #[clap(long, default_value = ".")]
    pub manifest_path: PathBuf,
    /// Open the docs in a browser after building them.
    #[clap(long)]
    pub open: bool,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_doc::doc(command)?;
    Ok(())
}
