use fuels::accounts::wallet::WalletUnlocked;
use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageVecNestedContract",
    abi = "test_projects/storage_vec_nested/out/release/storage_vec_nested-abi.json",
));

async fn test_storage_vec_nested_instance() -> TestStorageVecNestedContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/storage_vec_nested/out/release/storage_vec_nested.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    TestStorageVecNestedContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn nested_vec_access_push() {
    let methods = test_storage_vec_nested_instance().await.methods();

    methods.nested_vec_access_push().call().await.unwrap();
}

#[tokio::test]
async fn nested_vec_access_insert() {
    let methods = test_storage_vec_nested_instance().await.methods();

    methods.nested_vec_access_insert().call().await.unwrap();
}
