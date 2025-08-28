use forc_debug::{ContractId, FuelClient, Transaction};

#[tokio::main]
async fn main() {
    run_example().await.expect("Running example failed");
}

async fn run_example() -> Result<(), anyhow::Error> {
    let client = FuelClient::new("http://localhost:4000/graphql")?;

    let session_id = client.start_session().await?;

    client
        .set_breakpoint(&session_id, ContractId::zeroed(), 0)
        .await?;

    let tx: Transaction =
        serde_json::from_str(include_str!("example_tx.json")).expect("Invalid transaction JSON");
    let status = client.start_tx(&session_id, &tx).await?;
    assert!(status.breakpoint.is_some());

    let value = client.register(&session_id, 12).await?;
    println!("reg[12] = {value}");

    let mem = client.memory(&session_id, 0x10, 0x20).await?;
    println!("mem[0x10..0x30] = {mem:?}");

    client.set_single_stepping(&session_id, true).await?;

    let status = client.continue_tx(&session_id).await?;
    assert!(status.breakpoint.is_some());

    client.set_single_stepping(&session_id, false).await?;

    let status = client.continue_tx(&session_id).await?;
    assert!(status.breakpoint.is_none());

    client.end_session(&session_id).await?;

    Ok(())
}
