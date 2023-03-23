use fuels::prelude::*;

abigen!(Contract(
    name = "TestExperimentalStorageNestedMapsContract",
    abi = "test_projects/experimental_storage_nested_maps/out/debug/experimental_storage_nested_maps-abi.json",
));

async fn test_experimental_storage_nested_maps_instance(
) -> TestExperimentalStorageNestedMapsContract {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/experimental_storage_nested_maps/out/debug/experimental_storage_nested_maps.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/experimental_storage_nested_maps/out/debug/experimental_storage_nested_maps-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();

    TestExperimentalStorageNestedMapsContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn nested_map_1_access() {
    let methods = test_experimental_storage_nested_maps_instance()
        .await
        .methods();

    methods.nested_map_1_access().call().await.unwrap();
}

#[tokio::test]
async fn nested_map_2_access() {
    let methods = test_experimental_storage_nested_maps_instance()
        .await
        .methods();

    methods.nested_map_2_access().call().await.unwrap();
}

#[tokio::test]
async fn nested_map_3_access() {
    let methods = test_experimental_storage_nested_maps_instance()
        .await
        .methods();

    methods.nested_map_3_access().call().await.unwrap();
}
