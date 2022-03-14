#![allow(warnings)]
mod cli;
mod ops;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run_cli().await
}
