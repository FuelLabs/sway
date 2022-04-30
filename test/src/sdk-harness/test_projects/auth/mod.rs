use fuel_tx::{ContractId, Salt};
use fuels::prelude::*;
use fuels::signers::wallet::Wallet;
use fuels_abigen_macro::abigen;

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
        .returns_msg_sender_address(wallet.address())
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
        .set_contracts(&[auth_id])
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
    Wallet,
) {
    let salt = Salt::from([0u8; 32]);
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let compiled_1 = Contract::load_sway_contract(
        "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin",
        salt,
    )
    .unwrap();
    let compiled_2 = Contract::load_sway_contract(
        "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract.bin",
        salt,
    )
    .unwrap();

    let id_1 = Contract::deploy(&compiled_1, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    let id_2 = Contract::deploy(&compiled_2, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance_1 = AuthContract::new(id_1.to_string(), provider.clone(), wallet.clone());
    let instance_2 = AuthCallerContract::new(id_2.to_string(), provider.clone(), wallet.clone());

    (instance_1, id_1, instance_2, id_2, wallet)
}
