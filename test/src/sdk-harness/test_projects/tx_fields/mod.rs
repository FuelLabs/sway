use fuel_tx::{Bytes32, ContractId, Salt};
use fuel_types::bytes::WORD_SIZE;
use fuel_vm::consts::VM_TX_MEMORY;
use fuels_abigen_macro::abigen;
use fuels_contract::contract::Contract;
use fuels_contract::parameters::TxParameters;
use fuels_signers::util::test_helpers::setup_test_provider_and_wallet;
use fuels_signers::wallet::Wallet;
use fuels_signers::Signer;

abigen!(
    TxContractTest,
    "test_artifacts/tx_contract/out/debug/tx_contract-abi.json",
);

async fn get_contracts() -> (TxContractTest, ContractId, Wallet) {
    let salt = Salt::from([0u8; 32]);
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let compiled =
        Contract::load_sway_contract("test_artifacts/tx_contract/out/debug/tx_contract.bin", salt)
            .unwrap();

    let contract_id = Contract::deploy(&compiled, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    let instance = TxContractTest::new(contract_id.to_string(), provider.clone(), wallet.clone());

    (instance, contract_id, wallet)
}

#[tokio::test]
async fn can_get_tx_type() {
    let (contract_instance, _, _) = get_contracts().await;

    let result = contract_instance.get_tx_type().call().await.unwrap();
    // Script transactions are of type = 0
    assert_eq!(result.value, 0);
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
async fn can_get_byte_price() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports spending coins.
    let byte_price = 0;

    let result = contract_instance
        .get_tx_byte_price()
        .tx_params(TxParameters::new(None, None, Some(byte_price)))
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, byte_price);
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
    let script_length = 24;

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
    let script_data_length = 80;

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
    let zero_receipts_root = Bytes32::default();

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
    let script_start_offset = VM_TX_MEMORY + TRANSACTION_SCRIPT_FIXED_SIZE;

    let result = contract_instance
        .get_tx_script_start_offset()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, script_start_offset as u64);
}

#[tokio::test]
async fn can_get_tx_input_type() {
    let (contract_instance, _, _) = get_contracts().await;

    // Contract input
    let input_type = 1;
    let result_ptr = contract_instance
        .get_tx_input_pointer(0)
        .call()
        .await
        .unwrap();
    let result = contract_instance
        .get_tx_input_type(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, input_type);

    // Coin input
    let input_type = 0;
    let result_ptr = contract_instance
        .get_tx_input_pointer(1)
        .call()
        .await
        .unwrap();
    let result = contract_instance
        .get_tx_input_type(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, input_type);
}

#[tokio::test]
async fn can_get_tx_input_coin_owner() {
    let (contract_instance, _, wallet) = get_contracts().await;

    // Coin input
    let input_owner = txcontracttest_mod::Address {
        value: wallet.address().into(),
    };
    let result_ptr = contract_instance
        .get_tx_input_pointer(1)
        .call()
        .await
        .unwrap();
    let result = contract_instance
        .get_tx_input_coin_owner(result_ptr.value)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, input_owner);
}

#[tokio::test]
async fn can_get_tx_output_type() {
    let (contract_instance, _, _) = get_contracts().await;

    // Contract output
    let output_type = 1;
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
    assert_eq!(result.value, output_type);

    // Change output
    let output_type = 3;
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
    assert_eq!(result.value, output_type);
}
