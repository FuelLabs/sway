use fuels::prelude::*;

abigen!(Contract(
    name = "RunExternalProxyContract",
    abi = "test_projects/run_external_proxy/out/release/run_external_proxy-abi.json",
));

#[tokio::test]
async fn run_external_can_proxy_call() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let id = Contract::load_from(
        "test_projects/run_external_proxy/out/release/run_external_proxy.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let target_id = Contract::load_from(
        "test_projects/run_external_target/out/release/run_external_target.bin",
        LoadConfiguration::default()
            .with_storage_configuration(StorageConfiguration::default().with_autoload(false)),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = RunExternalProxyContract::new(id.clone(), wallet);

    let result = instance
        .methods()
        .foobar(target_id.clone())
        .with_contract_ids(&[target_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 42);
}
