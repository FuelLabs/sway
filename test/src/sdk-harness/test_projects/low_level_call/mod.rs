use fuels::{
    prelude::*,
    tx::ContractId,
    tx::{Bytes32, Input, Output, TxPointer, UtxoId},
};

// Load abi from json
abigen!(
    TestContract,
    "test_artifacts/low_level_callee_contract/out/debug/test_contract-abi.json"
);

script_abigen!(
    TestScript,
    "test_projects/low_level_call/out/debug/test_script-abi.json"
);



async fn low_level_call(id: ContractId, wallet: WalletUnlocked, function_selector: Vec<u8>, calldata: Vec<u8>, single_value_type_arg: bool) {
    // Build the script instance
    let script_instance = TestScript::new(
        wallet,
        "test_projects/low_level_call/out/debug/test_script.bin",
    );

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
        .main(id, function_selector, calldata, single_value_type_arg)
        .with_inputs(vec![contract_input])
        .with_outputs(vec![contract_output])
        .tx_params(TxParameters::new(None, Some(10_000_000), None));
    
    tx.call().await.unwrap();
}

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
            "test_artifacts/low_level_callee_contract/out/debug/test_contract-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = TestContract::new(id.clone(), wallet.clone());

    (instance, id.into(), wallet)
}

#[tokio::test]
async fn can_call_with_one_word_arg() {

    let (instance, id, wallet) = get_contract_instance().await;
    
    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/fn_selector_encoding.md#function-selector-encoding=
    // sha256("set_value(u64)")
    // hex : 00 00 00 00 e0  ff  38 8f
    // dec : 00 00 00 00 224 255 56 143
    let function_selector = vec![0u8, 0u8, 0u8, 0u8, 224u8, 255u8, 56u8, 143u8];
    
    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/argument_encoding.md
    // calldata is 42u64 (8 bytes)
    let calldata = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8];
    low_level_call(id, wallet, function_selector, calldata, true).await;

    // Calling "set_value(u64)" with argument "42" should set the value to 42
    let result = instance.methods().get_value().call().await.unwrap().value;
    assert_eq!(result, 42);
}

#[tokio::test]
async fn can_call_with_multi_word_arg() {
    let (instance, id, wallet) = get_contract_instance().await;

    // sha256("set_b256_value(b256)")
    // 0x6c 0x5e 0x2f 0xe2
    // 108 94 47 226
    let function_selector = vec![0u8, 0u8, 0u8, 0u8, 108u8, 94u8, 47u8, 226u8];

    // 0x0101010101010101010101010101010101010101010101010101010101010101
    let calldata = vec![1u8; 32];
    low_level_call(id, wallet, function_selector, calldata, false).await;
    let result = instance.methods().get_b256_value().call().await.unwrap().value;
    assert_eq!(result, fuels::core::types::Bits256([1u8; 32]));
} 

#[tokio::test]
async fn can_call_with_multiple_args() {
    let (instance, id, wallet) = get_contract_instance().await;

    let function_selector = vec![0u8, 0u8, 0u8, 0u8, 112u8, 224u8, 73u8, 19u8];
    let calldata = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 23u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8];
    low_level_call(id, wallet, function_selector, calldata, false).await;

    let result = instance.methods().get_value().call().await.unwrap().value;
    assert_eq!(result, 23+42);
}

#[tokio::test]
async fn can_call_with_multiple_args_complex() {
    let (instance, id, wallet) = get_contract_instance().await;
    
    // sha256("set_value_multiple_complex(s(bool,a[u64;3]),str[4])") 
    // 0x62 0xc3 0x1a 0x4c
    // 00 00 00 00 98 195 26 76
    let function_selector = vec![0u8, 0u8, 0u8, 0u8, 98u8, 195u8, 26u8, 76u8];
    let calldata = vec![
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, // true for MyStruct.a
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, // 1u64 for MyStruct.b[0]
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, // 2u64 for MyStruct.b[1]
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 3u8, // 3u64 for MyStruct.b[2]
        102u8, 117u8, 101u8, 108u8, 0u8, 0u8, 0u8, 0u8 // "fuel" (0x6675656c) for str[4]  (note right padding)
        ];
    low_level_call(id, wallet, function_selector, calldata, false).await;

    let result_uint = instance.methods().get_value().call().await.unwrap().value;
    let result_bool = instance.methods().get_bool_value().call().await.unwrap().value;
    let result_str = instance.methods().get_str_value().call().await.unwrap().value;
    
    assert_eq!(result_uint, 2);
    assert_eq!(result_bool, true);
    assert_eq!(result_str, "fuel");

}
