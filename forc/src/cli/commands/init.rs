use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project.
#[derive(Debug, Parser)]
pub struct Command {
    /// The path at which to create the manifest
    #[clap(long)]
    pub path: Option<String>,
    /// Create a package with a binary target (src/main.sw). This is the default behavior
    #[clap(long)]
    pub script: bool,
    /// Create a package with a library target (src/lib.sw).
    #[clap(long)]
    pub library: bool,
    /// Create a package with a contract target (src/contract.rs).
    #[clap(long)]
    pub contract: bool,
    /// Create a package with a contract target (src/predicate.rs).
    #[clap(long)]
    pub predicate: bool,
    /// Use verbose output.
    #[clap(short = 'v', long = "verbose")]
    pub verbose: bool,
    /// Set the package name. Defaults to the directory name
    #[clap(long = "name")]
    pub name: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_init::init(command)?;
    Ok(())
}
