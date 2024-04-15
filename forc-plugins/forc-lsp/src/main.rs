//! A simple `forc` plugin for starting the sway language server.
//!
//! Once installed and available via `PATH`, can be executed via `forc lsp`.

// Use Jemalloc for main binary
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    name = "forc-lsp",
    about = "Forc plugin for the Sway LSP (Language Server Protocol) implementation",
    version
)]
struct App {}

#[tokio::main]
async fn main() {
    App::parse();
    sway_lsp::start().await
}
