use fuels::prelude::*;

abigen!(Contract(
    name = "TestStorageInitContract",
    abi = "test_projects/storage_init/out/debug/storage_init-abi.json",
));

async fn test_storage_init_instance() -> TestStorageInitContract<WalletUnlocked> {
    let wallet = launch_provider_and_get_wallet().await;
    let id = Contract::deploy(
        "test_projects/storage_init/out/debug/storage_init.bin",
        &wallet,
        DeployConfiguration::default().set_storage_configuration(
            StorageConfiguration::default().set_storage_path(
                "test_projects/storage_init/out/debug/storage_init-storage_slots.json".to_string(),
            ),
        ),
    )
    .await
    .unwrap();

    TestStorageInitContract::new(id.clone(), wallet)
}

#[tokio::test]
async fn test_initializers() {
    let methods = test_storage_init_instance().await.methods();
    assert!(methods.test_initializers().call().await.unwrap().value);
}
