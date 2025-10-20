use fuels::prelude::*;

#[tokio::test]
async fn emits_indexed_events() -> Result<()> {
    abigen!(Script(
        name = "Events",
        abi = "test_projects/events/out/release/events-abi.json",
    ));

    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let bin_path = "test_projects/events/out/release/events.bin";
    let instance = Events::new(wallet.clone(), bin_path);

    let response = instance.main().call().await?;

    // TODO: Uncomment once fuels-rs is updated with indexed events support (https://github.com/FuelLabs/fuels-rs/pull/1695).
    // let events = response.decode_logs_with_type::<TestIndexedEventStruct>()?;
    // assert_eq!(events.len(), 3);
    // let flags: Vec<bool> = events.iter().map(|event| event.field_1).collect();
    // assert_eq!(flags, vec![true, false, true]);

    // let expected =
    //     hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a").unwrap();
    // assert_eq!(expected, response.tx_status.receipts[0].data().unwrap());

    Ok(())
}
