use forc_util::{init_tracing_subscriber, TracingSubscriberOptions, TracingWriterMode};
use tower_lsp::{LspService, Server};

mod capabilities;
mod core;
mod server;
pub mod utils;
use server::Backend;
use tracing::metadata::LevelFilter;
use utils::debug::DebugFlags;

pub async fn start(config: DebugFlags) {
    let tracing_options = TracingSubscriberOptions {
        log_level: Some(LevelFilter::DEBUG), // TODO: Set this based on IDE config
        writer_mode: Some(TracingWriterMode::Stderr),
        ..Default::default()
    };
    init_tracing_subscriber(tracing_options);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend::new(client, config))
        .custom_method("sway/runnables", Backend::runnables)
        .custom_method("sway/show_ast", Backend::show_ast)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
