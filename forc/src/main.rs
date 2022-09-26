use tracing::error;

#[tokio::main]
async fn main() {
    if let Err(err) = forc::cli::run_cli().await {
        error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
