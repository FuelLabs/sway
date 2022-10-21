#![recursion_limit = "256"]

use forc_util::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use tower_lsp::{LspService, Server};

mod capabilities;
mod core;
pub mod error;
mod server;
#[cfg(test)]
pub mod test_utils;
pub mod utils;
use server::Backend;
use tracing::metadata::LevelFilter;
use utils::debug::DebugFlags;

pub async fn start() {
    let tracing_options = TracingSubscriberOptions {
        log_level: Some(LevelFilter::DEBUG), // TODO: Set this based on IDE config
        writer_mode: Some(TracingWriterMode::Stderr),
        ..Default::default()
    };
    init_tracing_subscriber(tracing_options);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new)
        .custom_method("sway/runnables", Backend::runnables)
        .custom_method("sway/show_ast", Backend::show_ast)
        .custom_method("textDocument/inlayHint", Backend::inlay_hints)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
