#![recursion_limit = "256"]

use tower_lsp::{LspService, Server};

mod capabilities;
mod core;
pub mod error;
mod server;
#[cfg(test)]
pub mod test_utils;
pub mod utils;
use server::Backend;

pub async fn start() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new)
        .custom_method("sway/runnables", Backend::runnables)
        .custom_method("sway/show_ast", Backend::show_ast)
        .custom_method("textDocument/inlayHint", Backend::inlay_hints)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
