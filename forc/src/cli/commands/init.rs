use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project in an existing directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// The directory in which the forc project will be initialized.
    #[clap(long)]
    pub path: Option<String>,
    /// Create a package with a script target (src/main.sw).
    #[clap(long)]
    pub script: bool,
    /// Create a package with a library target (src/lib.sw). This is the default behavior.
    #[clap(long)]
    pub library: bool,
    /// Create a package with a contract target (src/contract.rs).
    #[clap(long)]
    pub contract: bool,
    /// Create a package with a predicate target (src/predicate.rs).
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
