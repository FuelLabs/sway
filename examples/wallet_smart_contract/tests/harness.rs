use fuels::prelude::*;
use fuels_abigen_macro::abigen;

// Generate Rust bindings from our contract JSON ABI
abigen!(MyContract, "examples/wallet_smart_contract/out/debug/wallet_smart_contract-abi.json");

#[tokio::test]
async fn harness() {
    // Launch a local network and deploy the contract
    let compiled = Contract::load_sway_contract("./out/debug/wallet_smart_contract.bin").unwrap();
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let contract_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    println!("Contract deployed @ {:x}", contract_id);

    let contract_instance = MyContract::new(contract_id.to_string(), provider, wallet.clone());

    let address = wallet.address();

    // withdraw some tokens to wallet
    contract_instance
        .send_funds(1_000_000, address)
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();
    
    assert!(true);
}
