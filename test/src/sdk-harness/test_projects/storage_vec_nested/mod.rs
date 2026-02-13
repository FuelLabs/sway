use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageVecNestedContract",
    abi = "out_for_sdk_harness_tests/storage_vec_nested-abi.json",
));

async fn test_storage_vec_nested_instance() -> TestStorageVecNestedContract<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/storage_vec_nested.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

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

#[tokio::test]
#[should_panic]
async fn revert_on_load_storage_vec() {
    let methods = test_storage_vec_nested_instance().await.methods();

    methods.revert_on_load_storage_vec().call().await.unwrap();
}
