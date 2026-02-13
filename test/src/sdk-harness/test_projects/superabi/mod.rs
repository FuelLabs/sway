use fuels::prelude::*;

abigen!(Contract(
    name = "SuperAbiTestContract",
    abi = "out_for_sdk_harness_tests/superabi-abi.json"
));

async fn get_superabi_instance() -> SuperAbiTestContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/superabi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    SuperAbiTestContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn abi_test() -> Result<()> {
    let instance = get_superabi_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.abi_test().call().await?;
    assert_eq!(42, response.value);

    Ok(())
}

#[tokio::test]
async fn superabi_test() -> Result<()> {
    let instance = get_superabi_instance().await;
    let contract_methods = instance.methods();

    let response = contract_methods.superabi_test().call().await?;
    assert_eq!(41, response.value);

    Ok(())
}
