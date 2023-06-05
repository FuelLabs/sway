use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageMapNestedContract",
    abi = "test_projects/storage_map_nested/out/debug/storage_map_nested-abi.json",
));

async fn test_storage_map_nested_instance() -> TestStorageMapNestedContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::load_from(
        "test_projects/storage_map_nested/out/debug/storage_map_nested.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    TestStorageMapNestedContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn nested_map_1_access() {
    let methods = test_storage_map_nested_instance().await.methods();

    methods.nested_map_1_access().call().await.unwrap();
}

#[tokio::test]
async fn nested_map_2_access() {
    let methods = test_storage_map_nested_instance().await.methods();

    methods.nested_map_2_access().call().await.unwrap();
}

#[tokio::test]
async fn nested_map_3_access() {
    let methods = test_storage_map_nested_instance().await.methods();

    methods.nested_map_3_access().call().await.unwrap();
}
