mod cli;
mod manifest;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    cli::run_cli()
}
