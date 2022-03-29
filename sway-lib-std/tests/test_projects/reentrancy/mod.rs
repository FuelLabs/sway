use fuel_tx::Salt;
use fuels_abigen_macro::abigen;
use fuels_contract::contract::Contract;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

abigen!(
    TestFuelCoinContract,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);
abigen!(AttackerContract, "test_artifacts/reentrancy_attacker_contract/src/abi.json",);

#[tokio::test]
#[ignore]
async fn not_reentrant() {
}

#[tokio::test]
async fn is_reentrant() {
    let salt = Salt::from([0u8; 32]);
    let compiled =
        Contract::load_sway_contract("test_projects/token_ops/out/debug/token_ops.bin", salt)
            .unwrap();
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let id = Contract::deploy(&compiled, &client).await.unwrap();
    let fuelcoin_instance = TestFuelCoinContract::new(id.to_string(), client);
}

#[tokio::test]
#[ignore]
async fn script_usage_of_is_reentrant() {
}
