use tower_lsp::{LspService, Server};

mod capabilities;
mod core;
mod server;
mod sway_config;
mod utils;
use server::Backend;

pub async fn start() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
