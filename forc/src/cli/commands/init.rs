use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project in an existing directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// The directory in which the forc project will be initialized.
    #[clap(long)]
    pub path:      Option<String>,
    /// The default program type, excluding all flags or adding this flag creates a basic contract program.
    #[clap(long)]
    pub contract:  bool,
    /// Create a package with a script target (src/main.sw).
    #[clap(long)]
    pub script:    bool,
    /// Create a package with a predicate target (src/predicate.rs).
    #[clap(long)]
    pub predicate: bool,
    /// Create a package with a library target (src/lib.sw).
    #[clap(long)]
    pub library:   bool,
    /// Set the package name. Defaults to the directory name
    #[clap(long)]
    pub name:      Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_init::init(command)?;
    Ok(())
}
