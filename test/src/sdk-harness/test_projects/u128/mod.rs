use fuel_tx::{ContractId, Salt};
use fuels::prelude::*;
use fuels::signers::wallet::Wallet;
use fuels_abigen_macro::abigen;

// Load abi from json
abigen!(U128Contract, "test_projects/u128/out/debug/u128-abi.json");

async fn get_contract_instance() -> (U128Contract, ContractId) {
    // Deploy the compiled contract
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/u128/out/debug/u128.bin", salt).unwrap();

    // Launch a local network and deploy the contract
    let (provider, wallet) = setup_test_provider_and_wallet().await;

    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = U128Contract::new(id.to_string(), provider, wallet);

    (instance, id)
}

async fn test_u64mul(
    instance: &U128Contract,
    a: u64,
    b: u64,
    expected_upper: u64,
    expected_lower: u64,
) {
    let result = instance.multiply_u64(a, b).call().await.unwrap();

    assert_eq!(result.value.0, expected_upper);
    assert_eq!(result.value.1, expected_lower);
}

#[tokio::test]
async fn multiply_u64() {
    let (u128_instance, id) = get_contract_instance().await;

    test_u64mul(&u128_instance, 2, 5, 0, 10).await;
    test_u64mul(&u128_instance, 0xabcd, 0, 0, 0).await;
    test_u64mul(&u128_instance, 0xabcd, 1, 0, 0xabcd).await;
    test_u64mul(
        &u128_instance,
        0xffffffffffffffff,
        0x0000000000000100,
        0x00000000000000ff,
        0xffffffffffffff00,
    )
    .await;
    test_u64mul(
        &u128_instance,
        0xabababababababab,
        0xbabababababababa,
        0x7d37f2ad6822dd97,
        0x589de3286db2f83e,
    )
    .await;
}
