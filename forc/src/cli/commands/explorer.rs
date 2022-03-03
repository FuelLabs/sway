use crate::ops::forc_explorer;
use anyhow::{bail, Result};
use clap::Parser;

/// Run the network explorer.
#[derive(Debug, Parser)]
pub struct Command {
    /// The port number
    #[clap(short = 'p', long = "port", default_value = "3030")]
    pub port: String,
    #[clap(subcommand)] // Note that we mark a field as a subcommand
    pub clean: Option<CleanCommand>,
}

#[derive(Debug, Parser)]
pub enum CleanCommand {
    Clean,
}

pub(crate) async fn exec(_command: Command) -> Result<()> {
    match forc_explorer::exec(_command).await {
        Err(e) => bail!(e),
        _ => Ok(()),
    }
}
