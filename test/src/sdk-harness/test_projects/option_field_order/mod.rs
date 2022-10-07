use fuels::prelude::*;

abigen!(
    MyContract,
    "test_projects/option_field_order/out/debug/option_field_order-abi.json"
);

#[tokio::test]
async fn default_is_none() {
    let instance = setup().await;
    assert!(instance.methods().is_none().call().await.unwrap().value);
}

async fn setup() -> MyContract {
    let wallet = launch_provider_and_get_wallet().await;

    let id = Contract::deploy(
        "test_projects/option_field_order/out/debug/option_field_order.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/option_field_order/out/debug/option_field_order-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = MyContract::new(id.to_string(), wallet);

    instance
}
