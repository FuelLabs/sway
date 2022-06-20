use fuels::prelude::*;
use fuels::signers::wallet::Wallet;
use fuels::tx::{default_parameters::MAX_GAS_PER_TX, ContractId};
use fuels_abigen_macro::abigen;

abigen!(
    MethodsContract,
    "test_artifacts/methods_contract/out/debug/methods_abi-abi.json",
);

#[tokio::test]
async fn test_function() {
    let wallet = launch_provider_and_get_single_wallet().await;
    let (instance, _) = get_methods_instance(wallet).await;

    let result = instance
        .test_function()
        .call()
        .await
        .unwrap();
    assert_eq!(result, true);
}

async fn get_methods_instance(wallet: Wallet) -> (MethodsContract, ContractId) {
    let id = Contract::deploy(
        "test_artifacts/methods_contract/out/debug/methods_contract.bin",
        &wallet,
        TxParameters::default(),
    )
    .await
    .unwrap();

    let instance = MethodsContract::new(id.to_string(), wallet);

    (instance, id)
}
