use fuels::prelude::*;

abigen!(Contract(
    name = "MethodsContract",
    abi = "out/methods_contract-abi.json",
));

#[tokio::test]
async fn run_methods_test() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let instance = get_methods_instance(wallet).await;

    let result = instance
        .methods()
        .test_function()
        .with_tx_policies(TxPolicies::default().with_script_gas_limit(2757))
        .call()
        .await
        .unwrap();

    // Increase the limit above and uncomment the line below to see how many gas is being used
    // run with --nocapture
    // dbg!(&result);

    assert!(result.value);
}

async fn get_methods_instance(wallet: Wallet) -> MethodsContract<Wallet> {
    let id = Contract::load_from(
        "out/methods_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    MethodsContract::new(id.clone(), wallet)
}
