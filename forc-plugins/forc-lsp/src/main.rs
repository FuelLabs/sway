//! A simple `forc` plugin for starting the sway language server.
//!
//! Once installed and available via `PATH`, can be executed via `forc lsp`.

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    name = "forc-lsp",
    about = "Forc plugin for the Sway LSP (Language Server Protocol) implementation.",
    version
)]
struct App {
    /// Instructs the client to draw squiggly lines under all of the tokens that our server managed
    /// to parse. Expects either "typed" or "parsed".
    #[clap(long)]
    pub collected_tokens_as_warnings: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = App::parse();
    let dbg = sway_lsp::utils::debug::DebugFlags {
        collected_tokens_as_warnings: app.collected_tokens_as_warnings,
    };
    sway_lsp::start(dbg).await
}
