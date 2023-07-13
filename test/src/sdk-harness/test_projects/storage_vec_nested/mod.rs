use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageVecNestedContract",
    abi = "test_projects/storage_vec_nested/out/debug/storage_vec_nested-abi.json",
));

async fn test_storage_vec_nested_instance() -> TestStorageVecNestedContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::load_from(
        "test_projects/storage_vec_nested/out/debug/storage_vec_nested.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    TestStorageVecNestedContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn nested_vec_access() {
    let methods = test_storage_vec_nested_instance().await.methods();

    methods.nested_vec_access().call().await.unwrap();
}
