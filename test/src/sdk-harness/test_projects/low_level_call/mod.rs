use fuels::{prelude::*, tx::{Bytes32, Input, Output, TxPointer, UtxoId}, tx::ContractId};

// Load abi from json
abigen!(TestContract, "test_artifacts/low_level_callee_contract/out/debug/test_contract-abi.json");

script_abigen!(TestScript, "test_projects/low_level_call/out/debug/test_script-abi.json");

async fn get_contract_instance() -> (TestContract, ContractId, WalletUnlocked) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    let wallet = wallets.pop().unwrap();

    let id = Contract::deploy(
        "test_artifacts/low_level_callee_contract/out/debug/test_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_artifacts/low_level_callee_contract/out/debug/test_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = TestContract::new(id.clone(), wallet.clone());

    (instance, id.into(), wallet)
}

#[tokio::test]
async fn can_call_contract_with_generic_call() {

    // Get the contract ID
    let (_instance, id, wallet) = get_contract_instance().await;

    // Build the script instance
    let script_instance = TestScript::new(wallet, "test_projects/low_level_call/out/debug/test_script.bin");


    // Add the contract being called to the inputs and outputs
    let contract_input = Input::Contract {
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
        tx_pointer: TxPointer::default(),
        contract_id: id,
    };

    let contract_output = Output::Contract {
        input_index: 0u8,
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),      
    };
    
    // Run the script which will call the contract


    let tx = script_instance
    .main(id)
    .with_inputs(vec![contract_input])
    .with_outputs(vec![contract_output]);

    println!("Inputs: \n{:?}\n", tx.script_call.inputs);
    println!("Outputs: \n{:?}\n", tx.script_call.outputs);


    let result = tx
    .call()
    .await
    .unwrap();

    println!("Result: {:?}", result);

    // Display return value
    println!("Result: {}", result.value);
}
