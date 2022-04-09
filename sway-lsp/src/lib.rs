use tower_lsp::{LspService, Server};

mod capabilities;
mod core;
mod server;
mod sway_config;
pub mod utils;
use server::Backend;
use utils::debug::DebugFlags;

pub async fn start(config: DebugFlags) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client, config));
    Server::new(stdin, stdout, socket).serve(service).await;
}
