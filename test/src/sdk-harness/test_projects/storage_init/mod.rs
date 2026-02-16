use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageInitContract",
    abi = "out/storage_init-abi.json",
));

async fn test_storage_init_instance() -> TestStorageInitContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out/storage_init.bin",
        LoadConfiguration::default().with_storage_configuration(
            StorageConfiguration::default()
                .add_slot_overrides_from_file(
                    "out/storage_init-storage_slots.json",
                )
                .unwrap(),
        ),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestStorageInitContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn test_initializers() {
    let methods = test_storage_init_instance().await.methods();
    assert!(methods.test_initializers().call().await.unwrap().value);
    // let l = methods.test_initializers().call().await;
    // let (receipts, value) = match l {
    //     Ok(l) => (l.receipts, l.value),
    //     Err(Error::RevertTransactionError { receipts, .. }) => receipts,
    //     _ => todo!(),
    // };
    // pretty_assertions::assert_eq!(&receipts[4].data(), &receipts[5].data());
    // assert!(value);
}
