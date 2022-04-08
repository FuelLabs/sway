use crate::ops::forc_lsp;
use anyhow::{bail, Result};
use clap::Parser;

/// Run the LSP server.
#[derive(Debug, Parser)]
pub struct Command {
    /// Instructs the client to draw squiggly lines
    /// under all of the tokens that our server managed to parse
    #[clap(long)]
    pub parsed_tokens_as_warnings: bool,
}

pub(crate) async fn exec(command: Command) -> Result<()> {
    match forc_lsp::exec(command).await {
        Err(e) => bail!(e),
        _ => Ok(()),
    }
}
