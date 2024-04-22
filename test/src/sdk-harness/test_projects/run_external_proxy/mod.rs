use fuels::prelude::*;

abigen!(Contract(
    name = "RunExternalProxyContract",
    abi = "test_projects/run_external_proxy/out/release/run_external_proxy-abi.json",
));

#[tokio::test]
async fn run_external_can_proxy_call() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let target_id = Contract::load_from(
        "test_projects/run_external_target/out/release/run_external_target.bin",
        LoadConfiguration::default()
            .with_storage_configuration(StorageConfiguration::default().with_autoload(false)),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let configurables = RunExternalProxyContractConfigurables::default()
        .with_TARGET(target_id.clone().into())
        .unwrap();
    let id = Contract::load_from(
        "test_projects/run_external_proxy/out/release/run_external_proxy.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = RunExternalProxyContract::new(id.clone(), wallet);

    // Call "double_value"
    // Will call run_external_proxy::double_value
    // that will call run_external_target::double_value
    // and return the value doubled.
    let result = instance
        .methods()
        .double_value(42)
        .with_contract_ids(&[target_id.clone().into()])
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 84);

    // Call "does_not_exist_in_the_target"
    // Will call run_external_proxy::does_not_exist_in_the_target
    // it will proxy the call to run_external_target,
    // and endup in the fallback, fn that will triple the input value
    let result = instance
        .methods()
        .does_not_exist_in_the_target(42)
        .with_contract_ids(&[target_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, 126);
}
