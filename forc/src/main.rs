#![allow(warnings)]
mod cli;
mod ops;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    match cli::run_cli().await {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    };
    Ok(())
}
