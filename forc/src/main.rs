#![allow(warnings)]
mod cli;
mod ops;
mod utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    cli::run_cli().await
}
