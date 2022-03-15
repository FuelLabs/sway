use anyhow::Result;
use clap::Parser;

/// Run the LSP server.
#[derive(Debug, Parser)]
pub(crate) struct Command {}

pub(crate) async fn exec(_command: Command) -> Result<()> {
    sway_lsp::start().await;
    Ok(())
}
