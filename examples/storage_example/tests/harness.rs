use fuels::prelude::*;
use fuels_abigen_macro::abigen;

// Generate Rust bindings from our contract JSON ABI
abigen!(MyContract, "examples/storage_example/out/debug/storage_example-abi.json");

#[tokio::test]
async fn harness() {
    // Launch a local network and deploy the contract
    let compiled = Contract::load_sway_contract("./out/debug/storage_example.bin").unwrap();
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let contract_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    println!("Contract deployed @ {:x}", contract_id);

    let contract_instance = MyContract::new(contract_id.to_string(), provider, wallet);

    // Call `store_something()` method in our deployed contract.
    contract_instance
        .store_something(18)
        .call()
        .await
        .unwrap();

    // Call `get_something()` method in our deployed contract.
    let result = contract_instance
        .get_something()
        .call()
        .await
        .unwrap();

    assert_eq!(18, result.value);
}
