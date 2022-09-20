use fuel_vm::fuel_crypto::Hasher;
use fuel_vm::fuel_tx::ConsensusParameters;
use fuel_vm::fuel_tx::Transaction as FuelTransaction;
use fuel_vm::fuel_asm::Opcode;
use fuel_vm::consts::REG_ONE;
use fuels::prelude::*;
use fuels::tx::{Bytes32, ContractId, Contract as TxContract, Input as TxInput, TxPointer, UtxoId};
use std::str::FromStr;

abigen!(
    TxContractTest,
    "test_artifacts/tx_contract/out/debug/tx_contract-abi.json",
);

async fn get_contracts() -> (TxContractTest, ContractId, WalletUnlocked) {
    let mut provider_config = Config::local_node();
    provider_config.predicates = true;
    let wallet_config = WalletsConfig::new(Some(1), None, None);
    let mut wallets = launch_custom_provider_and_get_wallets(wallet_config, Some(provider_config)).await;
    let wallet = wallets.pop().unwrap();

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

async fn generate_coin_predicate_input(amount: u64, data: Vec<u8>, wallet: &WalletUnlocked) -> (Vec<u8>, TxInput) {
    let mut predicate_bytecode = Opcode::RET(REG_ONE).to_bytes().to_vec();
    predicate_bytecode.append(&mut predicate_bytecode.clone());
    predicate_bytecode.append(&mut predicate_bytecode.clone());
    predicate_bytecode.append(&mut predicate_bytecode.clone());
    let predicate_root: [u8; 32] = (*TxContract::root_from_code(&predicate_bytecode)).into();
    let predicate_root = Address::from(predicate_root);

    let provider = wallet.get_provider().unwrap();

    let _receipt = wallet
        .transfer(
            &predicate_root.into(),
            amount,
            AssetId::default(),
            TxParameters::default(),
        )
        .await
        .unwrap();

    let predicate_coin = &provider.get_coins(&predicate_root.into(), AssetId::default()).await.unwrap()[0];
    let predicate_coin = TxInput::CoinPredicate {
        utxo_id: UtxoId::from(predicate_coin.utxo_id.clone()),
        owner: Address::from(predicate_coin.owner.clone()),
        amount: predicate_coin.amount.clone().into(),
        asset_id: AssetId::from(predicate_coin.asset_id.clone()),
        tx_pointer: TxPointer::default(),
        maturity: 0,
        predicate: predicate_bytecode.clone(),
        predicate_data: data.clone(),
    };

    (predicate_bytecode, predicate_coin)
}

#[tokio::test]
async fn can_get_tx_type() {
    let (contract_instance, _, _) = get_contracts().await;

    let result = contract_instance.get_tx_type().call().await.unwrap();
    // Script transactions are of type = 0
    assert_eq!(result.value, Transaction::Script());
}

#[tokio::test]
async fn can_get_tx_gas_price() {
    let (contract_instance, _, _) = get_contracts().await;
    let gas_price = 3;

    let result = contract_instance
        .get_tx_gas_price()
        .tx_params(TxParameters::new(Some(gas_price), None, None))
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, gas_price);
}

#[tokio::test]
async fn can_get_tx_gas_limit() {
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
async fn can_get_tx_maturity() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports setting maturity.
    let maturity = 0;

    let result = contract_instance.get_tx_maturity().call().await.unwrap();
    assert_eq!(result.value, maturity);
}

#[tokio::test]
async fn can_get_tx_script_length() {
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
async fn can_get_tx_script_data_length() {
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
async fn can_get_input_count() {
    let (contract_instance, _, _) = get_contracts().await;
    let inputs_count = 2;

    let result = contract_instance
        .get_input_count()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, inputs_count);
}

#[tokio::test]
async fn can_get_output_count() {
    let (contract_instance, _, _) = get_contracts().await;
    let outputs_count = 2;

    let result = contract_instance
        .get_output_count()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, outputs_count);
}

#[tokio::test]
async fn can_get_tx_witnesses_count() {
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
async fn can_get_tx_receipts_root() {
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
async fn can_get_tx_script_start_offset() {
    let (contract_instance, _, _) = get_contracts().await;

    let script_start_offset = ConsensusParameters::DEFAULT.tx_offset()
        + fuel_vm::fuel_tx::consts::TRANSACTION_SCRIPT_FIXED_SIZE;

    let result = contract_instance
        .get_tx_script_start_pointer()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, script_start_offset as u64);
}

#[tokio::test]
async fn can_get_script_bytecode_hash() {
    let (contract_instance, _, _) = get_contracts().await;

    let tx = contract_instance
        .get_tx_script_bytecode_hash()
        .get_call_execution_script()
        .await
        .unwrap()
        .tx;
    let hash = match tx {
        FuelTransaction::Script { script, .. } => {
            // Make sure script is actually something fairly substantial
            assert!(script.len() > 1);
            Hasher::hash(&script)
        }
        _ => Hasher::hash(&vec![]),
    };

    let result = contract_instance
        .get_tx_script_bytecode_hash()
        .call()
        .await
        .unwrap();
    assert_eq!(result.value.to_vec(), hash.to_vec());
}

#[tokio::test]
async fn can_get_input_type() {
    let (contract_instance, _, _) = get_contracts().await;

    let result = contract_instance.get_input_type(0).call().await.unwrap();
    assert_eq!(result.value, Input::Contract());

    let result = contract_instance.get_input_type(1).call().await.unwrap();

    assert_eq!(result.value, Input::Coin());
}

#[tokio::test]
async fn can_get_input_owner() {
    let (contract_instance, _, wallet) = get_contracts().await;

    let owner_result = contract_instance
        .get_input_owner(1)
        .call()
        .await
        .unwrap();
    assert_eq!(owner_result.value, wallet.address().into());

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_input_amount() {
    let (contract_instance, _, _) = get_contracts().await;
    let amount = 1000000000;

    let amount_result = contract_instance
        .get_input_amount(1)
        .call()
        .await
        .unwrap();
    assert_eq!(amount_result.value, amount);

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_input_asset() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports setting maturity.

    let asset_result = contract_instance
        .get_input_asset(1)
        .call()
        .await
        .unwrap();
    assert_eq!(asset_result.value, ContractId::zeroed());

    //TODO: add test for inputmessage
    //let asset_id = ContractId::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c").unwrap();
}

#[tokio::test]
async fn can_get_input_maturity() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports setting maturity.
    let maturity = 0;

    let result = contract_instance.get_input_maturity(1).call().await.unwrap();
    assert_eq!(result.value, maturity);
}

#[tokio::test]
async fn can_get_input_witness_index() {
    let (contract_instance, _, _) = get_contracts().await;
    // TODO set this to a non-zero value once SDK supports multiple witnesses.
    let witness_index = 0;

    let result = contract_instance.get_input_witness_index(1).call().await.unwrap();
    assert_eq!(result.value, witness_index);

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_input_predicate_length() {
    let (contract_instance, _, wallet) = get_contracts().await;
    let provider = wallet.get_provider().unwrap();
    let (predicate_bytecode, predicate_coin) = generate_coin_predicate_input(100, vec![], &wallet).await;

    // Add predicate coin to inputs and call contract
    let call_handler = contract_instance.get_input_predicate_length(2);
    let mut script = call_handler.get_call_execution_script().await.unwrap();
    if let FuelTransaction::Script { inputs, .. } = &mut script.tx {
        inputs.push(predicate_coin)
    }
    let result = call_handler.get_response(script.call(provider).await.unwrap()).unwrap();
    assert_eq!(result.value, predicate_bytecode.len() as u16);

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_input_predicate() {
    let (contract_instance, _, wallet) = get_contracts().await;
    let provider = wallet.get_provider().unwrap();
    let (predicate_bytecode, predicate_coin) = generate_coin_predicate_input(100, vec![], &wallet).await;
    let predicate_bytes: [u8; 32] = predicate_bytecode.try_into().unwrap();

    // Add predicate coin to inputs and call contract
    let call_handler = contract_instance.get_input_predicate(2);
    let mut script = call_handler.get_call_execution_script().await.unwrap();
    if let FuelTransaction::Script { inputs, .. } = &mut script.tx {
        inputs.push(predicate_coin)
    }
    let result = call_handler.get_response(script.call(provider).await.unwrap()).unwrap();
    assert_eq!(result.value, predicate_bytes);

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_input_predicate_data_length() {
    let (contract_instance, _, wallet) = get_contracts().await;
    let provider = wallet.get_provider().unwrap();
    let predicate_data_bytes: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let predicate_data = predicate_data_bytes.to_vec();
    let (_, predicate_coin) = generate_coin_predicate_input(100, predicate_data.clone(), &wallet).await;

    // Add predicate coin to inputs and call contract
    let call_handler = contract_instance.get_input_predicate_data_length(2);
    let mut script = call_handler.get_call_execution_script().await.unwrap();
    if let FuelTransaction::Script { inputs, .. } = &mut script.tx {
        inputs.push(predicate_coin)
    }
    let result = call_handler.get_response(script.call(provider).await.unwrap()).unwrap();
    assert_eq!(result.value, predicate_data.len() as u16);

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_input_predicate_data() {
    let (contract_instance, _, wallet) = get_contracts().await;
    let provider = wallet.get_provider().unwrap();
    let predicate_data_bytes: [u8; 32] = Bytes32::from_str("49a7e58dc8b6397a25de9090cb50e71c2588c773487d1da7066d0c7198c567c0").unwrap().into();
    let predicate_data = predicate_data_bytes.to_vec();
    let (_, predicate_coin) = generate_coin_predicate_input(100, predicate_data.clone(), &wallet).await;

    // Add predicate coin to inputs and call contract
    let call_handler = contract_instance.get_input_predicate_data(2);
    let mut script = call_handler.get_call_execution_script().await.unwrap();
    if let FuelTransaction::Script { inputs, .. } = &mut script.tx {
        inputs.push(predicate_coin)
    }
    let result = call_handler.get_response(script.call(provider).await.unwrap()).unwrap();
    assert_eq!(result.value, predicate_data_bytes);

    //TODO: add test for inputmessage
}

#[tokio::test]
async fn can_get_output_type() {
    let (contract_instance, _, _) = get_contracts().await;
    let result = contract_instance
        .get_output_type(0)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Output::Contract());

    let result = contract_instance
        .get_output_type(1)
        .call()
        .await
        .unwrap();
    assert_eq!(result.value, Output::Change());
}

#[tokio::test]
async fn can_get_tx_id() {
    let (contract_instance, _, _) = get_contracts().await;

    let call_handler = contract_instance.get_tx_id();
    let script = call_handler.get_call_execution_script().await.unwrap();
    let tx_id = script.tx.id();

    let result = contract_instance.get_tx_id().call().await.unwrap();

    let byte_array: [u8; 32] = tx_id.into();

    assert_eq!(result.value, byte_array);
}
