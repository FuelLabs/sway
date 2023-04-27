#![recursion_limit = "256"]

mod capabilities;
pub mod config;
mod core;
pub mod error;
pub mod server;
mod traverse;
pub mod utils;

use server::Backend;
use tower_lsp::{LspService, Server};

pub async fn start() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new)
        .custom_method("sway/show_ast", Backend::show_ast)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
