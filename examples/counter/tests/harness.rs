use fuels::prelude::*;
use fuels_abigen_macro::abigen;

// Generate Rust bindings from our contract JSON ABI
abigen!(MyContract, "examples/counter/out/debug/counter-abi.json");

#[tokio::test]
async fn harness() {
    // Launch a local network and deploy the contract
    let compiled = Contract::load_sway_contract("./out/debug/counter.bin").unwrap();
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let contract_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    println!("Contract deployed @ {:x}", contract_id);

    let contract_instance = MyContract::new(contract_id.to_string(), provider, wallet);

    // Call `initialize_counter()` method in our deployed contract.
    // Note that, here, you get type-safety for free!
    let result = contract_instance
        .initialize_counter(42)
        .call()
        .await
        .unwrap();

    assert_eq!(42, result.value);

    // Call `increment_counter()` method in our deployed contract.
    let result = contract_instance
        .increment_counter(10)
        .call()
        .await
        .unwrap();

    assert_eq!(52, result.value);
}
