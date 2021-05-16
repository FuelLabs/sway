#![allow(warnings)]
mod cli;
mod ops;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    match cli::run_cli() {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    };
    Ok(())
}
