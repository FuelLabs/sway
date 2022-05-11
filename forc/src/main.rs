use std::env;
use std::str::FromStr;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;

const LOG_FILTER: &str = "RUST_LOG";
const HUMAN_LOGGING: &str = "HUMAN_LOGGING";

#[tokio::main]
async fn main() {
    let filter = match env::var_os(LOG_FILTER) {
        Some(_) => EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    let human_logging = env::var_os(HUMAN_LOGGING)
        .map(|s| {
            bool::from_str(s.to_str().unwrap())
                .expect("Expected `true` or `false` to be provided for `HUMAN_LOGGING`")
        })
        .unwrap_or(true);

    let sub = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter);

    if human_logging {
        sub.with_ansi(true)
            .with_level(true)
            .with_line_number(true)
            .init();
    } else {
        sub.with_ansi(false)
            .with_level(true)
            .with_line_number(true)
            .json()
            .init();
    }
    if let Err(_err) = forc::cli::run_cli().await {
        error!("Forc error!");
    }
}
