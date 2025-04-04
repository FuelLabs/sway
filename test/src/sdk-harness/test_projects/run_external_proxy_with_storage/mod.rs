use fuels::{prelude::*, types::Bits256};

abigen!(Contract(
    name = "RunExternalProxyContract",
    abi = "test_projects/run_external_proxy_with_storage/out/release/run_external_proxy_with_storage-abi.json",
));

#[tokio::test]
async fn run_external_can_proxy_call() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let storage_configuration =
        StorageConfiguration::default().add_slot_overrides_from_file("test_projects/run_external_target_with_storage/out/release/run_external_target_with_storage-storage_slots.json").unwrap();

    let target_id = Contract::load_from(
        "test_projects/run_external_target_with_storage/out/release/run_external_target_with_storage.bin",
        LoadConfiguration::default()
            .with_storage_configuration(storage_configuration.clone()),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let configurables = RunExternalProxyContractConfigurables::default()
        .with_TARGET(target_id.clone().into())
        .unwrap();
    let id = Contract::load_from(
        "test_projects/run_external_proxy_with_storage/out/release/run_external_proxy_with_storage.bin",
        LoadConfiguration::default().with_configurables(configurables).with_storage_configuration(storage_configuration),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;
    let instance = RunExternalProxyContract::new(id.clone(), wallet);
    // Call "large_value"
    // Will call run_external_proxy::large_value
    // that will call run_external_target::large_value
    // and return the value doubled.
    let result = instance
        .methods()
        .large_value()
        .with_contract_ids(&[target_id.clone().into()])
        .call()
        .await
        .unwrap();
    for r in result.tx_status.receipts.iter() {
        match r {
            Receipt::LogData { data, .. } => {
                if let Some(data) = data {
                    if data.len() > 8 {
                        if let Ok(s) = std::str::from_utf8(&data[8..]) {
                            print!("{:?} ", s);
                        }
                    }
                    println!("{:?}", data);
                }
            }
            _ => {}
        }
    }
    let expected_large =
        Bits256::from_hex_str("0x00000000000000000000000059F2f1fCfE2474fD5F0b9BA1E73ca90b143Eb8d0")
            .unwrap();
    assert_eq!(result.value, expected_large);
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
    for r in result.tx_status.receipts.iter() {
        match r {
            Receipt::LogData { data, .. } => {
                if let Some(data) = data {
                    if data.len() > 8 {
                        if let Ok(s) = std::str::from_utf8(&data[8..]) {
                            print!("{:?} ", s);
                        }
                    }
                    println!("{:?}", data);
                }
            }
            _ => {}
        }
    }
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
