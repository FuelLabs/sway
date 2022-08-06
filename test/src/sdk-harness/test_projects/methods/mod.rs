use fuels::prelude::*;
use fuels::signers::wallet::Wallet;

abigen!(
    MethodsContract,
    "test_artifacts/methods_contract/out/debug/methods_contract-abi.json",
);

#[tokio::test]
async fn run_methods_test() {
    let wallet = launch_provider_and_get_wallet().await;
    let instance = get_methods_instance(wallet).await;

    let result = instance.test_function().call().await.unwrap();
    assert_eq!(result.value, true);
}

async fn get_methods_instance(wallet: Wallet) -> MethodsContract {
    let id = Contract::deploy(
        "test_artifacts/methods_contract/out/debug/methods_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_artifacts/methods_contract/out/debug/methods_contract-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();
    MethodsContractBuilder::new(id.to_string(), wallet).build()
}
