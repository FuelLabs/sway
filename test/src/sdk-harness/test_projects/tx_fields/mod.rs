use fuel_types::bytes::WORD_SIZE;
use fuel_vm::fuel_tx::ConsensusParameters;
use fuels::contract::contract::ContractCallHandler;
use fuels::prelude::*;
use fuels::signers::wallet::Wallet;
use fuels::tx::{Bytes32, ContractId};
use std::str::FromStr;

abigen!(
    TxContractTest,
    "test_artifacts/tx_contract/out/debug/tx_contract-abi.json",
);

async fn get_contracts() -> (TxContractTest, ContractId, Wallet) {
    let wallet = launch_provider_and_get_wallet().await;

    let contract_id = Contract::deploy(
        "test_artifacts/tx_contract/out/debug/tx_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_artifacts/tx_contract/out/debug/tx_contract-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();
    let instance = TxContractTestBuilder::new(contract_id.to_string(), wallet.clone()).build();

    (instance, contract_id.into(), wallet)
}

#[tokio::test]
async fn can_get_tx_type() {
    let (contract_instance, _, _) = get_contracts().await;

    let result = contract_instance.get_tx_type().call().await.unwrap();
    // Script transactions are of type = 0
    assert_eq!(result.value, Transaction::Script());
}

#[tokio::test]
async fn can_get_gas_price() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports spending coins.
    let gas_price = 0;

    let result = contract_instance
        .get_tx_gas_price()
        .tx_params(TxParameters::new(Some(gas_price), None, None))
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, gas_price);
}

#[tokio::test]
async fn can_get_gas_limit() {
    let (contract_instance, _, _) = get_contracts().await;
    let gas_limit = 420301;

    let result = contract_instance
        .get_tx_gas_limit()
        .tx_params(TxParameters::new(None, Some(gas_limit), None))
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, gas_limit);
}

#[tokio::test]
async fn can_get_maturity() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports setting maturity.
    let maturity = 0;

    let result = contract_instance.get_tx_maturity().call().await.unwrap();
    assert_eq!(result.value, maturity);
}

#[tokio::test]
async fn can_get_script_length() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO use programmatic script length https://github.com/FuelLabs/fuels-rs/issues/181
    let script_length = 32;

    let result = contract_instance
        .get_tx_script_length()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, script_length);
}

#[tokio::test]
async fn can_get_script_data_length() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO make this programmatic.
    let script_data_length = 88;

    let result = contract_instance
        .get_tx_script_data_length()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, script_data_length);
}

#[tokio::test]
async fn can_get_inputs_count() {
    let (contract_instance, _, _) = get_contracts().await;
    let inputs_count = 2;

    let result = contract_instance
        .get_tx_inputs_count()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, inputs_count);
}

#[tokio::test]
async fn can_get_outputs_count() {
    let (contract_instance, _, _) = get_contracts().await;
    let outputs_count = 2;

    let result = contract_instance
        .get_tx_outputs_count()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, outputs_count);
}

#[tokio::test]
async fn can_get_witnesses_count() {
    let (contract_instance, _, _) = get_contracts().await;
    let witnesses_count = 1;

    let result = contract_instance
        .get_tx_witnesses_count()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, witnesses_count);
}

#[tokio::test]
async fn can_get_receipts_root() {
    let (contract_instance, _, _) = get_contracts().await;
    let zero_receipts_root =
        Bytes32::from_str("4be973feb50f1dabb9b2e451229135add52f9c0973c11e556fe5bce4a19df470")
            .unwrap();

    let result = contract_instance
        .get_tx_receipts_root()
        .call()
        .await
        .unwrap();
    assert_ne!(Bytes32::from(result.value), zero_receipts_root);
}

#[tokio::test]
async fn can_get_script_start_offset() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO https://github.com/FuelLabs/fuel-tx/issues/98
    const TRANSACTION_SCRIPT_FIXED_SIZE: usize = WORD_SIZE // Identifier
    + WORD_SIZE // Gas price
    + WORD_SIZE // Gas limit
    + WORD_SIZE // Byte price
    + WORD_SIZE // Maturity
    + WORD_SIZE // Script size
    + WORD_SIZE // Script data size
    + WORD_SIZE // Inputs size
    + WORD_SIZE // Outputs size
    + WORD_SIZE // Witnesses size
    + Bytes32::LEN; // Receipts root
    let script_start_offset =
        ConsensusParameters::DEFAULT.tx_offset() + TRANSACTION_SCRIPT_FIXED_SIZE;

    let result = contract_instance
        .get_tx_script_start_pointer()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, script_start_offset as u64);
}

#[tokio::test]
async fn can_get_tx_input_type() {
    let (contract_instance, _, _) = get_contracts().await;

    let result_ptr = contract_instance
        .get_tx_input_pointer(0)
        .call()
        .await
        .unwrap();

    let result = contract_instance
        .get_tx_input_type_from_ptr(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Input::Contract());

    let result_ptr = contract_instance
        .get_tx_input_pointer(1)
        .call()
        .await
        .unwrap();
    let result = contract_instance
        .get_tx_input_type_from_ptr(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Input::Coin());
}

// TODO: Add tests for getting InputMessage owner, type when InputMessages land.
#[tokio::test]
async fn can_get_tx_input_coin_owner() {
    let (contract_instance, _, wallet) = get_contracts().await;

    let owner_result = contract_instance
        .get_tx_input_coin_owner(0)
        .call()
        .await
        .unwrap();

    assert_eq!(owner_result.value, wallet.address().into());
}

#[tokio::test]
async fn can_get_tx_output_type() {
    let (contract_instance, _, _) = get_contracts().await;

    let result_ptr = contract_instance
        .get_tx_output_pointer(0)
        .call()
        .await
        .unwrap();
    let result = contract_instance
        .get_tx_output_type(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Output::Contract());

    let result_ptr = contract_instance
        .get_tx_output_pointer(1)
        .call()
        .await
        .unwrap();
    let result = contract_instance
        .get_tx_output_type(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Output::Change());
}

#[tokio::test]
async fn can_get_tx_id() {
    let (contract_instance, _, _) = get_contracts().await;

    let call_handler = contract_instance.get_tx_id(0);
    let script = call_handler.get_call_execution_script().await.unwrap();
    let tx_id = script.tx.id();

    let result = contract_instance.get_tx_id(0).call().await.unwrap();

    let byte_array: [u8; 32] = tx_id.into();

    assert_eq!(result.value, Option::Some(byte_array));
}
