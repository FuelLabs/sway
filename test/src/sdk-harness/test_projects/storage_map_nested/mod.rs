use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageMapNestedContract",
    abi = "test_projects/storage_map_nested/out/release/storage_map_nested-abi.json",
));

async fn test_storage_map_nested_instance() -> TestStorageMapNestedContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "test_projects/storage_map_nested/out/release/storage_map_nested.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

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
