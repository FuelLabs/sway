use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    forc::cli::run_cli().await
}
