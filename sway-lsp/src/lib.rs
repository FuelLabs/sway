#![recursion_limit = "256"]

use tower_lsp::{LspService, Server};

mod capabilities;
pub mod config;
mod core;
pub mod error;
pub mod global_state;
mod handlers {
    pub mod notification;
    pub mod request;
}
pub mod lsp_ext;
pub mod server;
mod traverse;
pub mod utils;

use global_state::GlobalState;

pub async fn start() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(GlobalState::new)
        .custom_method("sway/show_ast", GlobalState::show_ast)
        .custom_method("textDocument/inlayHint", GlobalState::inlay_hints)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
