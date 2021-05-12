mod cli;
mod constants;
mod defaults;
mod manifest;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    match cli::run_cli() {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    };
    Ok(())
}
