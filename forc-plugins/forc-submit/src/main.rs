//! A `forc` plugin for submitting transactions to a Fuel network.

use anyhow::Context;
use clap::Parser;
use fuel_gql_client::client::FuelClient;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let cmd = forc_submit::Command::parse();
    let tx = forc_submit::read_tx(&cmd.tx_path)?;
    let client = FuelClient::new(&cmd.network.node_url)?;
    if cmd.network.await_ {
        let status = client
            .submit_and_await_commit(&tx)
            .await
            .context("Submission of tx or awaiting commit failed")?;
        if cmd.tx_status.json {
            forc_submit::print_status_json(&status)?;
        } else {
            forc_submit::print_status(&status);
        }
    } else {
        let id = client.submit(&tx).await.context("Failed to submit tx")?;
        println!("{id}");
    }
    Ok(())
}
