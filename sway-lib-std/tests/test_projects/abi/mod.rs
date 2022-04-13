use fuel_tx::{ContractId, Salt};
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, parameters::TxParameters};
use fuels_signers::util::test_helpers;
use fuel_core::service::Config;
use fuel_tx::Transaction;
use fuels_contract::script::Script;
use fuels_signers::provider::Provider;
use std::fs::read;

abigen!(
    Callee,
    "test_artifacts/abi_wrapper_testing_contract/out/debug/abi_wrapper_testing_conract-abi.json",
);


async fn execute_script(bin_path: &str, provider: Provider) -> u64 {
    let bin = read(bin_path);

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: 1_000_000,
        maturity: 0,
        byte_price: 0,
        receipts_root: Default::default(),
        script: bin.unwrap(), // Here we pass the compiled script into the transaction
        script_data: vec![],
        inputs: vec![],
        outputs: vec![],
        witnesses: vec![vec![].into()],
        metadata: None,
    };

    let script = Script::new(tx);
    let receipts = script.call(&provider).await.unwrap();

    receipts[0].val().unwrap()
}


#[tokio::test]
async fn test_abi_wrapper() {
    // Deploy callee
    let salt = Salt::from([0u8; 32]);
    let compiled_callee = Contract::load_sway_contract("test_artifacts/abi_wrapper_testing_contract/out/debug/abi.bin", salt).unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let _ = Contract::deploy(&compiled_callee, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    // Execute script and check value
    let return_val = execute_script("./out/debug/abi.bin", provider).await;
    assert_eq!(42, return_val);


}