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
struct App {}

#[tokio::main]
async fn main() {
    sway_lsp::start().await
}
