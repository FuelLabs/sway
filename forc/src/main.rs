use forc_util::set_subscriber;
use tracing::error;

#[tokio::main]
async fn main() {
    set_subscriber();
    if let Err(err) = forc::cli::run_cli().await {
        error!("Error: {:?}", err);
        std::process::exit(1);
    }
}
