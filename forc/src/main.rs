use forc_util::init_tracing_subscriber;
use tracing::error;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    if let Err(err) = forc::cli::run_cli().await {
        error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
