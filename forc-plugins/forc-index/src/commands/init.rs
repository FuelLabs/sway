use crate::ops::forc_index_init;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Create a new Forc project in the current directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// Name of index
    #[clap(long, help = "Name of index.")]
    pub name: Option<String>,

    /// Path at which to create index
    #[clap(
        short,
        long,
        parse(from_os_str),
        help = "Path at which to create index."
    )]
    pub path: Option<PathBuf>,

    /// Name of the index namespace
    #[clap(long, help = "Namespace in which index belongs.")]
    pub namespace: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_init::init(command)?;
    Ok(())
}
