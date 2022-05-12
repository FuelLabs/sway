use forc_util::set_subscriber;
use tracing::error;

#[tokio::main]
async fn main() {
    set_subscriber();
    if let Err(_err) = forc::cli::run_cli().await {
        error!("forc error!");
        std::process::exit(1);
    }
}
