use lspower::{LspService, Server};

mod capabilities;
mod core;
mod server;
mod utils;
use server::Backend;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}
