use crate::ops::forc_index_start;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Create a new Forc project in an existing directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// Log level passed to the Fuel Indexer service.
    #[clap(long, default_value = "info", value_parser(["info", "debug", "error", "warn"]), help = "Log level passed to the Fuel Indexer service.")]
    pub log_level: String,

    /// Path to the config file used to start the Fuel Indexer.
    #[clap(long, help = "Path to the config file used to start the Fuel Indexer.")]
    pub config: Option<PathBuf>,

    /// Path to the fuel-indexer binary.
    #[clap(long, help = "Path to the fuel-indexer binary.")]
    pub bin: Option<PathBuf>,

    /// Whether to run the Fuel Indexer in the background.
    #[clap(long, help = "Whether to run the Fuel Indexer in the background.")]
    pub background: bool,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_start::init(command)?;
    Ok(())
}
