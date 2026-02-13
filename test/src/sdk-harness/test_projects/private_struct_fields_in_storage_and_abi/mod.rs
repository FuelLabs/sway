use fuels::prelude::*;

abigen!(Contract(
    name = "TestPrivateStructFieldsInStorageAndAbi",
    abi = "out_for_sdk_harness_tests/private_struct_fields_in_storage_and_abi-abi.json",
));

async fn test_storage_private_struct_fields_instance(
) -> TestPrivateStructFieldsInStorageAndAbi<Wallet> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let id = Contract::load_from(
        "out_for_sdk_harness_tests/private_struct_fields_in_storage_and_abi.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    TestPrivateStructFieldsInStorageAndAbi::new(id.clone(), wallet)
}

#[tokio::test]
async fn read_initial_can_init_via_storage() {
    let methods = test_storage_private_struct_fields_instance()
        .await
        .methods();

    assert_eq!(
        methods
            .read_initial_can_init_via_storage()
            .call()
            .await
            .unwrap()
            .value,
        CanInitStruct { x: 11, y: 12 }
    );
}

#[tokio::test]
async fn write_and_read_can_init_via_storage() {
    let methods = test_storage_private_struct_fields_instance()
        .await
        .methods();

    let input = CanInitStruct { x: 1111, y: 2222 };

    assert_eq!(
        methods
            .write_and_read_can_init_via_storage(input.clone())
            .call()
            .await
            .unwrap()
            .value,
        input
    );
}

#[tokio::test]
async fn write_and_read_cannot_init_via_api() {
    let methods = test_storage_private_struct_fields_instance()
        .await
        .methods();

    let input = CannotInitStruct { x: 1111, y: 2222 };

    assert_eq!(
        methods
            .write_and_read_cannot_init_via_api(input.clone())
            .call()
            .await
            .unwrap()
            .value,
        input
    );
}
