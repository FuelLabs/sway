use tower_lsp::{LspService, Server};

mod capabilities;
mod core;
mod server;
pub mod utils;
use server::Backend;
use utils::debug::DebugFlags;

pub async fn start(config: DebugFlags) {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend::new(client, config))
        .custom_method("sway/runnables", Backend::runnables)
        .custom_method("sway/show_ast", Backend::show_ast)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
