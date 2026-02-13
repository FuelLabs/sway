use fuels::prelude::*;

abigen!(Contract(
    name = "MyContract",
    abi = "out_for_sdk_harness_tests/option_field_order-abi.json"
));

#[tokio::test]
async fn default_is_none() {
    let instance = setup().await;
    assert!(instance.methods().is_none().call().await.unwrap().value);
}

async fn setup() -> MyContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let id = Contract::load_from(
        "out_for_sdk_harness_tests/option_field_order.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    MyContract::new(id.clone(), wallet)
}
