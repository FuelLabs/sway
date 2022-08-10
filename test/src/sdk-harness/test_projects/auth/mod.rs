use fuels::{prelude::*, tx::ContractId};

abigen!(
    AuthContract,
    "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract-abi.json"
);
abigen!(
    AuthCallerContract,
    "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-abi.json"
);

#[tokio::test]
async fn is_external_from_sdk() {
    let (auth_instance, _, _, _, _) = get_contracts().await;
    let result = auth_instance.is_caller_external().call().await.unwrap();

    assert_eq!(result.value, true);
}

#[tokio::test]
async fn msg_sender_from_sdk() {
    let (auth_instance, _, _, _, wallet) = get_contracts().await;
    let result = auth_instance
        .returns_msg_sender_address(wallet.address().into())
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

#[tokio::test]
async fn msg_sender_from_contract() {
    let (_, auth_id, caller_instance, caller_id, _) = get_contracts().await;

    let result = caller_instance
        .call_auth_contract(auth_id, caller_id)
        .set_contracts(&[auth_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

async fn get_contracts() -> (
    AuthContract,
    ContractId,
    AuthCallerContract,
    ContractId,
    LocalWallet,
) {
    let wallet = launch_provider_and_get_wallet().await;

    let id_1 = Contract::deploy(
        "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(
            Some(
                "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract-storage_slots.json".to_string(),
                )
        )
    )
    .await
    .unwrap();

    let id_2 = Contract::deploy(
        "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();

    let instance_1 = AuthContractBuilder::new(id_1.to_string(), wallet.clone()).build();
    let instance_2 = AuthCallerContractBuilder::new(id_2.to_string(), wallet.clone()).build();

    (instance_1, id_1.into(), instance_2, id_2.into(), wallet)
}
