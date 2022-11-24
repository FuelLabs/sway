use fuel_vm::{consts::REG_ONE, fuel_asm::Opcode, fuel_crypto::Hasher};
use fuels::{
    prelude::*,
    tx::{
        field::Script as ScriptField, field::Witnesses, field::*, Bytes32, ConsensusParameters,
        Contract as TxContract, ContractId, Input as TxInput, Output as TxOutput, TxPointer,
        UniqueIdentifier, UtxoId,
    },
};
use std::str::FromStr;

const MESSAGE_DATA: [u8; 3] = [1u8, 2u8, 3u8];

abigen!(
    TxContractTest,
    "test_artifacts/tx_contract/out/debug/tx_contract-abi.json",
);

async fn get_contracts() -> (TxContractTest, ContractId, WalletUnlocked) {
    let mut wallet = WalletUnlocked::new_random(None);

    let coins = setup_single_asset_coins(wallet.address(), BASE_ASSET_ID, 1, 10);

    let messages = setup_single_message(
        &Bech32Address {
            hrp: "".to_string(),
            hash: Default::default(),
        },
        wallet.address(),
        DEFAULT_COIN_AMOUNT,
        69,
        MESSAGE_DATA.to_vec(),
    );

    let (provider, _address) =
        setup_test_provider(coins.clone(), messages.clone(), None, None).await;
    wallet.set_provider(provider);

    let contract_id = Contract::deploy(
        "test_artifacts/tx_contract/out/debug/tx_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let instance = TxContractTest::new(contract_id.clone(), wallet.clone());

    (instance, contract_id.into(), wallet)
}

async fn generate_predicate_inputs(
    amount: u64,
    data: Vec<u8>,
    wallet: &WalletUnlocked,
) -> (Vec<u8>, TxInput, TxInput) {
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

    let predicate_coin = &provider
        .get_coins(&predicate_root.into(), AssetId::default())
        .await
        .unwrap()[0];
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

    let message = &wallet.get_messages().await.unwrap()[0];
    let message_id = TxInput::compute_message_id(
        &message.sender.clone().into(),
        &message.recipient.clone().into(),
        message.nonce.clone().into(),
        message.amount.0,
        &data,
    );
    let predicate_message = TxInput::message_predicate(
        message_id,
        message.sender.clone().into(),
        message.recipient.clone().into(),
        message.amount.0,
        0,
        data.clone(),
        predicate_bytecode.clone(),
        data.clone(),
    );

    (predicate_bytecode, predicate_coin, predicate_message)
}

fn generate_outputs() -> Vec<TxOutput> {
    let mut v = vec![TxOutput::variable(Address::zeroed(), 0, AssetId::default())];
    v.push(TxOutput::message(Address::zeroed(), 0));
    v
}

mod tx {
    use super::*;

    #[tokio::test]
    async fn can_get_tx_type() {
        let (contract_instance, _, _) = get_contracts().await;

        let result = contract_instance
            .methods()
            .get_tx_type()
            .call()
            .await
            .unwrap();
        // Script transactions are of type = 0
        assert_eq!(result.value, Transaction::Script());
    }

    #[tokio::test]
    async fn can_get_gas_price() {
        let (contract_instance, _, _) = get_contracts().await;
        let gas_price = 3;

        let result = contract_instance
            .methods()
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
            .methods()
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

        let result = contract_instance
            .methods()
            .get_tx_maturity()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, maturity);
    }

    #[tokio::test]
    async fn can_get_script_length() {
        let (contract_instance, _, _) = get_contracts().await;
        // TODO use programmatic script length https://github.com/FuelLabs/fuels-rs/issues/181
        let script_length = 32;

        let result = contract_instance
            .methods()
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
            .methods()
            .get_tx_script_data_length()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, script_data_length);
    }

    #[tokio::test]
    async fn can_get_inputs_count() {
        let (contract_instance, _, _) = get_contracts().await;
        let inputs_count = 3;

        let result = contract_instance
            .methods()
            .get_tx_inputs_count()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, inputs_count);
    }

    #[tokio::test]
    async fn can_get_outputs_count() {
        let (contract_instance, _, _) = get_contracts().await;
        let outputs_count = 1;

        let result = contract_instance
            .methods()
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
            .methods()
            .get_tx_witnesses_count()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, witnesses_count);
    }

    #[tokio::test]
    async fn can_get_witness_pointer() {
        let (contract_instance, _, _) = get_contracts().await;

        let result = contract_instance
            .methods()
            .get_tx_witness_pointer(0)
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, 11144);
    }

    #[tokio::test]
    async fn can_get_witness_data_length() {
        let (contract_instance, _, _) = get_contracts().await;

        let result = contract_instance
            .methods()
            .get_tx_witness_data_length(0)
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, 64);
    }

    #[tokio::test]
    async fn can_get_witness_data() {
        let (contract_instance, _, _) = get_contracts().await;

        let call_handler = contract_instance.methods().get_tx_witness_data(0);
        let script = call_handler.get_call_execution_script().await.unwrap();
        let witnesses = script.tx.witnesses();

        let result = contract_instance
            .methods()
            .get_tx_witness_data(0)
            .call()
            .await
            .unwrap();

        let mut witness_first = witnesses[0].clone().into_inner();
        let witness_second = witness_first.split_off(32);

        let first_witness_array = witness_first.try_into().unwrap();
        let second_witness_array = witness_second.try_into().unwrap();
        let expected_b512 =
            B512::from((Bits256(first_witness_array), Bits256(second_witness_array)));

        assert_eq!(result.value, expected_b512);
    }

    #[tokio::test]
    async fn can_get_receipts_root() {
        let (contract_instance, _, _) = get_contracts().await;
        let zero_receipts_root =
            Bytes32::from_str("4be973feb50f1dabb9b2e451229135add52f9c0973c11e556fe5bce4a19df470")
                .unwrap();

        let result = contract_instance
            .methods()
            .get_tx_receipts_root()
            .call()
            .await
            .unwrap();
        assert_ne!(Bytes32::from(result.value.0), zero_receipts_root);
    }

    #[tokio::test]
    async fn can_get_script_start_offset() {
        let (contract_instance, _, _) = get_contracts().await;

        let script_start_offset = ConsensusParameters::DEFAULT.tx_offset()
            + fuel_vm::fuel_tx::consts::TRANSACTION_SCRIPT_FIXED_SIZE;

        let result = contract_instance
            .methods()
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
            .methods()
            .get_tx_script_bytecode_hash()
            .get_call_execution_script()
            .await
            .unwrap()
            .tx;

        let script = tx.script();
        let hash = if script.len() > 1 {
            Hasher::hash(&script)
        } else {
            Hasher::hash(&vec![])
        };

        let result = contract_instance
            .methods()
            .get_tx_script_bytecode_hash()
            .call()
            .await
            .unwrap();
        assert_eq!(Bytes32::from(result.value.0), hash);
    }

    #[tokio::test]
    async fn can_get_tx_id() {
        let (contract_instance, _, _) = get_contracts().await;

        let call_handler = contract_instance.methods().get_tx_id();
        let script = call_handler.get_call_execution_script().await.unwrap();
        let tx_id = script.tx.id();

        let result = contract_instance
            .methods()
            .get_tx_id()
            .call()
            .await
            .unwrap();

        let byte_array: [u8; 32] = tx_id.into();

        assert_eq!(result.value, Bits256(byte_array));
    }

    #[tokio::test]
    async fn can_get_get_tx_script_data_start_pointer() {
        let (contract_instance, _, _) = get_contracts().await;
        let result = contract_instance
            .methods()
            .get_tx_script_data_start_pointer()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, 10376)
    }
}

mod inputs {
    use super::*;

    mod revert {
        use super::*;

        mod contract {
            use super::*;

            #[tokio::test]
            #[should_panic(expected = "Revert(0)")]
            async fn fails_to_get_predicate_data_pointer_from_input_contract() {
                let (contract_instance, _, _) = get_contracts().await;
                let call_params = CallParameters::default();
                contract_instance
                    .methods()
                    .get_tx_input_predicate_data_pointer(0)
                    .call_params(call_params)
                    .call()
                    .await
                    .unwrap();
            }
        }
    }

    mod success {
        use super::*;

        #[tokio::test]
        async fn can_get_input_type() {
            let (contract_instance, _, _) = get_contracts().await;

            let result = contract_instance
                .methods()
                .get_input_type(0)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, Input::Contract());

            let result = contract_instance
                .methods()
                .get_input_type(1)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, Input::Message());
        }

        #[tokio::test]
        async fn can_get_tx_input_amount() {
            let default_amount = 1000000000;
            let (contract_instance, _, _) = get_contracts().await;
            let result = contract_instance
                .methods()
                .get_input_amount(1)
                .call()
                .await
                .unwrap();

            assert_eq!(result.value, default_amount);
        }

        #[tokio::test]
        async fn can_get_tx_input_coin_owner() {
            let (contract_instance, _, wallet) = get_contracts().await;

            let owner_result = contract_instance
                .methods()
                .get_input_owner(1)
                .call()
                .await
                .unwrap();

            assert_eq!(owner_result.value, wallet.address().into());
        }

        #[tokio::test]
        async fn can_get_input_predicate() {
            let (contract_instance, _, wallet) = get_contracts().await;
            let provider = wallet.get_provider().unwrap();
            let (predicate_bytecode, predicate_coin, predicate_message) =
                generate_predicate_inputs(100, vec![], &wallet).await;
            let predicate_bytes: [u8; 32] = predicate_bytecode.try_into().unwrap();

            // Add predicate coin to inputs and call contract
            let call_handler = contract_instance.methods().get_input_predicate(2);
            let script = call_handler.get_call_execution_script().await.unwrap();

            let new_outputs = generate_outputs();

            let inputs = script.tx.inputs();
            let outputs = script.tx.outputs();

            inputs.clone().push(predicate_coin);
            inputs.clone().push(predicate_message);
            for output in new_outputs {
                outputs.clone().push(output);
            }

            let result = call_handler
                .get_response(script.call(provider).await.unwrap())
                .unwrap();
            assert_eq!(result.value, Bits256(predicate_bytes));
        }

        mod message {
            use super::*;

            #[tokio::test]
            async fn can_get_input_message_msg_id() -> Result<(), Error> {
                let (contract_instance, _, wallet) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_message_msg_id(1)
                    .call()
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                let message_id: [u8; 32] = *messages[0].message_id.0 .0;
                ();
                assert_eq!(result.value, Bits256(message_id));
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_sender() -> Result<(), Error> {
                let (contract_instance, _, wallet) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_message_sender(1)
                    .call()
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                let contract_bytes: [u8; 32] = result.value.into();
                let message_bytes: [u8; 32] = messages[0].sender.0 .0.into();
                assert_eq!(contract_bytes, message_bytes);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_recipient() -> Result<(), Error> {
                let (contract_instance, _, wallet) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_message_recipient(1)
                    .call()
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                let contract_bytes: [u8; 32] = result.value.into();
                let message_bytes: [u8; 32] = messages[0].recipient.0 .0.into();
                assert_eq!(contract_bytes, message_bytes);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_nonce() -> Result<(), Error> {
                let (contract_instance, _, wallet) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_message_nonce(1)
                    .call()
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                // assert_eq!(result.value, messages[0].nonce);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_witness_index() -> Result<(), Error> {
                let (contract_instance, _, wallet) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_witness_index(1)
                    .call()
                    .await
                    .unwrap();
                // let messages = wallet.get_messages().await?;
                // assert_eq!(result.value, messages[0].witness_index);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_data_length() {
                let (contract_instance, _, _) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_message_data_length(1)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(result.value, 48u16);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_length() {
                let (contract_instance, _, _) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_predicate_length(1)
                    .append_variable_outputs(2)
                    .append_message_outputs(2)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(result.value, 0u16);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_data_length() {
                let (contract_instance, _, _) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_predicate_data_length(1)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(result.value, 0u16);
            }

            #[tokio::test]
            async fn can_get_input_message_data() -> Result<(), Error> {
                let (contract_instance, _, wallet) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_message_data(1, 0)
                    .call()
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                println!("data: {:?}", messages[0].data);
                println!("result data: {:?}", result.value);
                println!("MESSAGE_DATA: {:?}", MESSAGE_DATA);
                assert_eq!(result.value, MESSAGE_DATA);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_predicate() {
                let (contract_instance, _, wallet) = get_contracts().await;
                let provider = wallet.get_provider().unwrap();
                let (predicate_bytecode, predicate_coin, predicate_message) =
                    generate_predicate_inputs(100, vec![], &wallet).await;
                let predicate_bytes: [u8; 32] = predicate_bytecode.try_into().unwrap();

                // Add predicate coin to inputs and call contract
                let call_handler = contract_instance.methods().get_input_predicate(1);
                let script = call_handler.get_call_execution_script().await.unwrap();

                let new_outputs = generate_outputs();

                let inputs = script.tx.inputs();
                let outputs = script.tx.outputs();

                inputs.clone().push(predicate_coin);
                inputs.clone().push(predicate_message);
                for output in new_outputs {
                    outputs.clone().push(output);
                }

                let result = call_handler
                    .get_response(script.call(provider).await.unwrap())
                    .unwrap();
                assert_eq!(result.value, Bits256(predicate_bytes));
            }
        }
    }
}

mod outputs {
    use super::*;

    mod success {
        use super::*;

        #[tokio::test]
        async fn can_get_tx_output_type() {
            let (contract_instance, _, _) = get_contracts().await;
            let result = contract_instance
                .methods()
                .get_output_type(0)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, Output::Contract());
        }

        // TODO: test output.amount with outputs which have this field !
        // TODO: organize tests into modules
    }

    mod revert {
        use super::*;

        mod contract {
            use super::*;

            #[tokio::test]
            #[should_panic(expected = "Revert(0)")]
            async fn fails_to_get_amount_for_output_contract() {
                let (contract_instance, _, _) = get_contracts().await;
                contract_instance
                    .methods()
                    .get_tx_output_amount(0)
                    .call()
                    .await
                    .unwrap();
            }
        }
    }
}
