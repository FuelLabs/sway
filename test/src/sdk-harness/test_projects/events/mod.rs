use fuels::prelude::*;

#[tokio::test]
async fn run_valid() -> Result<()> {
    abigen!(Script(
        name = "Events",
        abi = "test/src/sdk-harness/test_projects/events/out/release/events-abi.json",
    ));

    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let bin_path = "test_projects/events/out/release/events.bin";
    let instance = Events::new(wallet.clone(), bin_path);

    let response = instance.main().call().await?;
    let log_event_struct = response.decode_logs_with_type::<TestEventStruct>()?;

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert_eq!(
        correct_hex.unwrap(),
        response.tx_status.receipts[0].data().unwrap()
    );

    Ok(())
}
