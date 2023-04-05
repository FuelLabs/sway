use fuels::prelude::*;

abigen!(Contract(
    name = "TestExperimentalStorageInitContract",
    abi = "test_projects/experimental_storage_init/out/debug/experimental_storage_init-abi.json",
));

async fn test_experimental_storage_init_instance(
) -> TestExperimentalStorageInitContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/experimental_storage_init/out/debug/experimental_storage_init.bin",
        &wallet,
        DeployConfiguration::default()
            .set_storage_configuration(StorageConfiguration::default().set_storage_path(
                "test_projects/experimental_storage_init/out/debug/experimental_storage_init-storage_slots.json".to_string()
            ))
    )
    .await
    .unwrap();

    TestExperimentalStorageInitContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn test_initializers() {
    let methods = test_experimental_storage_init_instance().await.methods();
    assert!(methods.test_initializers().call().await.unwrap().value);
}
