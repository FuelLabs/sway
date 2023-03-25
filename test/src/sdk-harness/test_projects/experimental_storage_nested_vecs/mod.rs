use fuels::prelude::*;

abigen!(Contract(
    name = "TestExperimentalStorageNestedVecsContract",
    abi = "test_projects/experimental_storage_nested_vecs/out/debug/experimental_storage_nested_vecs-abi.json",
));

async fn test_experimental_storage_nested_vecs_instance(
) -> TestExperimentalStorageNestedVecsContract {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/experimental_storage_nested_vecs/out/debug/experimental_storage_nested_vecs.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/experimental_storage_nested_vecs/out/debug/experimental_storage_nested_vecs-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();

    TestExperimentalStorageNestedVecsContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn nested_vec_1_access() {
    let methods = test_experimental_storage_nested_vecs_instance()
        .await
        .methods();

    methods.nested_vec_1_access().call().await.unwrap();
}
