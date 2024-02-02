use fuels::{accounts::wallet::WalletUnlocked, prelude::*};

abigen!(Contract(
    name = "MyContract",
    abi = "test_projects/option_field_order/out/release/option_field_order-abi.json"
));

#[tokio::test]
async fn default_is_none() {
    let instance = setup().await;
    assert!(instance.methods().is_none().call().await.unwrap().value);
}

async fn setup() -> MyContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let id = Contract::load_from(
        "test_projects/option_field_order/out/release/option_field_order.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    MyContract::new(id.clone(), wallet)
}
