use crate::cli::LspCommand;
use anyhow::Result;
use sway_lsp::utils::debug;

pub async fn exec(command: LspCommand) -> Result<()> {
    let config = debug::DebugFlags {
        parsed_tokens_as_warnings: command.parsed_tokens_as_warnings,
    };

    sway_lsp::start(config).await;

    Ok(())
}
