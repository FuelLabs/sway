use fuel_tx::{ContractId, Salt};
use fuels::prelude::*;
use fuels::test_helpers;
use fuels_abigen_macro::abigen;

abigen!(
    HashingTestContract,
    "test_projects/hashing/out/debug/hashing-abi.json"
);

async fn get_hashing_instance() -> (HashingTestContract, ContractId) {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/hashing/out/debug/hashing.bin", salt)
            .unwrap();
    let (provider, wallet) = test_helpers::setup_test_provider_and_wallet().await;
    let id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    let instance = HashingTestContract::new(id.to_string(), provider, wallet);

    (instance, id)
}

#[tokio::test]
async fn test_hash_u64() {
    let (instance, _id) = get_hashing_instance().await;
    // Check that hashing the same `u64` results in the same hash
    let result1 = instance.get_hash_u64(42).call().await.unwrap();
    let result2 = instance.get_hash_u64(42).call().await.unwrap();
    assert_eq!(result1.value, result2.value);
}
