use crate::ops::forc_clean;
use anyhow::Result;
use clap::Parser;

/// Removes the default forc compiler output artifact directory, i.e. `<project-name>/out`.
#[derive(Debug, Parser)]
pub struct Command {
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_clean::clean(command)
}
