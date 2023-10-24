use fuels::{
    accounts::wallet::{Wallet, WalletUnlocked},
    prelude::*,
    types::ContractId,
};

abigen!(
    Contract(
        name = "AuthContract",
        abi = "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract-abi.json"
    ),
    Contract(
        name = "AuthCallerContract",
        abi = "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-abi.json"
    )
);

#[tokio::test]
async fn is_external_from_sdk() {
    let (auth_instance, _, _, _, _) = get_contracts().await;
    let result = auth_instance
        .methods()
        .is_caller_external()
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

#[tokio::test]
async fn msg_sender_from_sdk() {
    let (auth_instance, _, _, _, wallet) = get_contracts().await;
    let result = auth_instance
        .methods()
        .returns_msg_sender_address(wallet.address())
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

#[tokio::test]
async fn msg_sender_from_contract() {
    let (auth_instance, auth_id, caller_instance, caller_id, _) = get_contracts().await;

    let result = caller_instance
        .methods()
        .call_auth_contract(auth_id, caller_id)
        .with_contracts(&[&auth_instance])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, true);
}

async fn get_contracts() -> (
    AuthContract<WalletUnlocked>,
    ContractId,
    AuthCallerContract<WalletUnlocked>,
    ContractId,
    Wallet,
) {
    let wallet = launch_provider_and_get_wallet().await;

    let id_1 = Contract::load_from(
        "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let id_2 = Contract::load_from(
        "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let instance_1 = AuthContract::new(id_1.clone(), wallet.clone());
    let instance_2 = AuthCallerContract::new(id_2.clone(), wallet.clone());

    (
        instance_1,
        id_1.into(),
        instance_2,
        id_2.into(),
        wallet.lock(),
    )
}
