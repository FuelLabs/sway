use anyhow::Result;
use clap::Parser;
use e2e_tests::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _ = run(cli).await;
    Ok(())
}
