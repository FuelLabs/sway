use fuel_tx::{ContractId, Salt};
use fuels::prelude::*;
use fuels::signers::wallet::Wallet;
use fuels_abigen_macro::abigen;

// Load abi from json
abigen!(U128Contract, "test_projects/u128/out/debug/u128-abi.json");

async fn get_contract_instance() -> (U128Contract, ContractId) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled = Contract::load_sway_contract("test_projects/u128/out/debug/u128.bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, wallet) = setup_test_provider_and_wallet().await;

    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = U128Contract::new(id.to_string(), provider, wallet);

    (instance, id)
}

#[tokio::test]
async fn multiply_u64() {
    let (u128_instance, id) = get_contract_instance().await;

    let a: u64 = u64::MAX;
    let b: u64 = 2;

    let result = u128_instance
    .multiply_u64(a, b)
    .call()
    .await
    .unwrap();

    // TO DO compare result to expected components

}