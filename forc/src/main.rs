#![allow(warnings)]
mod cli;
mod lock;
mod ops;
mod pkg;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run_cli().await
}
