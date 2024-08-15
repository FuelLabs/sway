mod e2e_vm_tests;
mod ir_generation;
mod reduced_std_libs;
mod test_consistency;

use anyhow::Result;
use clap::Parser;
use e2e_tests::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let _ = run(cli).await;
    Ok(())
}
