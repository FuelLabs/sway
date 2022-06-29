use crate::ops::forc_new;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project at <path>.
#[derive(Debug, Parser)]
pub struct Command {
    /// The default program type, excluding all flags or adding this flag creates a basic contract program.
    #[clap(long)]
    pub contract: bool,
    /// Adding this flag creates an empty script program.
    #[clap(long)]
    pub script: bool,
    /// Adding this flag creates an empty predicate program.
    #[clap(long)]
    pub predicate: bool,
    /// Adding this flag creates an empty library program.
    #[clap(long)]
    pub library: bool,
    /// The name of your project
    pub project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_new::init(command)?;
    Ok(())
}
