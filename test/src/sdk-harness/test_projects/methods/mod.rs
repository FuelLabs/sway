use fuels::prelude::*;

abigen!(Contract(
    name = "MethodsContract",
    abi = "test_artifacts/methods_contract/out/debug/methods_contract-abi.json",
));

#[tokio::test]
async fn run_methods_test() {
    let wallet = launch_provider_and_get_wallet().await;
    let instance = get_methods_instance(wallet).await;

    let result = instance.methods().test_function().call().await.unwrap();
    assert_eq!(result.value, true);
}

async fn get_methods_instance(wallet: WalletUnlocked) -> MethodsContract<WalletUnlocked> {
    let id = Contract::load_from(
        "test_artifacts/methods_contract/out/debug/methods_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();
    MethodsContract::new(id.clone(), wallet)
}
