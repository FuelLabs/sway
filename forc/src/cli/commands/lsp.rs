use anyhow::Result;
use clap::Parser;
use sway_server::start;
/// Run the LSP server.
#[derive(Debug, Parser)]
pub(crate) struct Command {}

pub(crate) async fn exec(_command: Command) -> Result<()> {
    start().await;
    Ok(())
}
