use fuel_vm::{consts::REG_ONE, fuel_asm::Opcode, fuel_crypto::Hasher};
use fuels::{
    contract::execution_script::ExecutableFuelCall,
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

async fn get_contracts() -> (TxContractTest, ContractId, WalletUnlocked, WalletUnlocked) {
    let mut wallet = WalletUnlocked::new_random(None);
    let mut deployment_wallet = WalletUnlocked::new_random(None);

    let mut deployment_coins = setup_single_asset_coins(
        deployment_wallet.address(),
        BASE_ASSET_ID,
        120,
        DEFAULT_COIN_AMOUNT,
    );

    let mut coins = setup_single_asset_coins(wallet.address(), BASE_ASSET_ID, 100, 1000);

    coins.append(&mut deployment_coins);

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

    wallet.set_provider(provider.clone());
    deployment_wallet.set_provider(provider);

    let contract_id = Contract::deploy(
        "test_artifacts/tx_contract/out/debug/tx_contract.bin",
        &deployment_wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let instance = TxContractTest::new(contract_id.clone(), deployment_wallet.clone());

    (instance, contract_id.into(), wallet, deployment_wallet)
}

async fn generate_predicate_inputs(
    amount: u64,
    data: Vec<u8>,
    wallet: &WalletUnlocked,
) -> (Vec<u8>, TxInput, TxInput) {
    // let mut predicate_bytecode = Opcode::RET(REG_ONE).to_bytes().to_vec();
    // predicate_bytecode.append(&mut predicate_bytecode.clone());
    // predicate_bytecode.append(&mut predicate_bytecode.clone());
    // predicate_bytecode.append(&mut predicate_bytecode.clone());
    // let predicate_root: [u8; 32] = (*TxContract::root_from_code(&predicate_bytecode)).into();
    // let predicate_root = Address::from(predicate_root);

    let predicate =
        Predicate::load_from("test_projects/tx_fields/out/debug/tx_predicate.bin").unwrap();

    let predicate_bytecode = predicate.code();
    let predicate_root = predicate.address();

    let provider = wallet.get_provider().unwrap();

    let _receipt = wallet
        .transfer(
            &predicate_root,
            amount,
            AssetId::default(),
            TxParameters::default(),
        )
        .await
        .unwrap();

    let predicate_coin = &provider
        .get_coins(&predicate_root, AssetId::default())
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
        message.amount,
        &data,
    );

    let predicate_message = TxInput::message_predicate(
        message_id,
        message.sender.clone().into(),
        message.recipient.clone().into(),
        message.amount,
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

async fn add_message_input(call: &mut ExecutableFuelCall, wallet: WalletUnlocked) {
    let message = &wallet.get_messages().await.unwrap()[0];

    let message_id = TxInput::compute_message_id(
        &message.sender.clone().into(),
        &message.recipient.clone().into(),
        message.nonce.clone().into(),
        message.amount,
        &message.data,
    );

    let message_input = TxInput::MessageSigned {
        message_id: message_id,
        sender: message.sender.clone().into(),
        recipient: message.recipient.clone().into(),
        amount: message.amount,
        nonce: message.nonce,
        witness_index: 0,
        data: message.data.clone(),
    };

    call.tx.inputs_mut().push(message_input);
}

mod tx {
    use super::*;

    #[tokio::test]
    async fn can_get_tx_type() {
        let (contract_instance, _, _, _) = get_contracts().await;

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
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, wallet, _) = get_contracts().await;

        let handler = contract_instance.methods().get_tx_inputs_count();
        let mut executable = handler.get_executable_call().await.unwrap();

        add_message_input(&mut executable, wallet.clone()).await;

        let inputs = executable.tx.inputs();

        let receipts = executable
            .execute(&wallet.get_provider().unwrap())
            .await
            .unwrap();

        assert_eq!(inputs.len() as u64, 3u64);
        assert_eq!(receipts[1].val().unwrap(), inputs.len() as u64);
    }

    #[tokio::test]
    async fn can_get_outputs_count() {
        let (contract_instance, _, _, _) = get_contracts().await;

        let call_handler = contract_instance.methods().get_tx_outputs_count();
        let script = call_handler.get_executable_call().await.unwrap();
        let outputs = script.tx.outputs();

        let result = contract_instance
            .methods()
            .get_tx_outputs_count()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, outputs.len() as u64);
    }

    #[tokio::test]
    async fn can_get_witnesses_count() {
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, _, _) = get_contracts().await;

        let result = contract_instance
            .methods()
            .get_tx_witness_pointer(0)
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, 10960);
    }

    #[tokio::test]
    async fn can_get_witness_data_length() {
        let (contract_instance, _, _, _) = get_contracts().await;

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
        let (contract_instance, _, wallet, _) = get_contracts().await;

        let handler = contract_instance.methods().get_tx_witness_data(0);
        let executable = handler.get_executable_call().await.unwrap();
        let witnesses = executable.tx.witnesses();

        let receipts = executable
            .execute(&wallet.get_provider().unwrap())
            .await
            .unwrap();

        assert_eq!(receipts[1].data().unwrap(), witnesses[0].as_vec());
    }

    #[tokio::test]
    async fn can_get_receipts_root() {
        let (contract_instance, _, _, _) = get_contracts().await;
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
        let (contract_instance, _, _, _) = get_contracts().await;

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
        let (contract_instance, _, _, _) = get_contracts().await;

        let tx = contract_instance
            .methods()
            .get_tx_script_bytecode_hash()
            .get_executable_call()
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
        let (contract_instance, _, wallet, _) = get_contracts().await;

        let handler = contract_instance.methods().get_tx_id();
        let executable = handler.get_executable_call().await.unwrap();
        let tx_id = executable.tx.id();

        let receipts = executable
            .execute(&wallet.get_provider().unwrap())
            .await
            .unwrap();
        let byte_array: [u8; 32] = tx_id.into();

        assert_eq!(receipts[1].data().unwrap(), byte_array);
    }

    #[tokio::test]
    async fn can_get_get_tx_script_data_start_pointer() {
        let (contract_instance, _, _, _) = get_contracts().await;
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
                let (contract_instance, _, _, _) = get_contracts().await;
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
            let (contract_instance, _, _, _) = get_contracts().await;

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
            assert_eq!(result.value, Input::Coin());
        }

        #[tokio::test]
        async fn can_get_tx_input_amount() {
            let default_amount = 1000000000;
            let (contract_instance, _, _, _) = get_contracts().await;
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
            let (contract_instance, _, _, deployment_wallet) = get_contracts().await;

            let owner_result = contract_instance
                .methods()
                .get_input_owner(1)
                .call()
                .await
                .unwrap();

            assert_eq!(owner_result.value, deployment_wallet.address().into());
        }

        #[tokio::test]
        async fn can_get_input_coin_predicate() {
            let (contract_instance, _, wallet, _) = get_contracts().await;
            let provider = wallet.get_provider().unwrap();
            let (predicate_bytecode, predicate_coin, predicate_message) =
                generate_predicate_inputs(100, vec![], &wallet).await;
            let predicate_bytes: Vec<u8> = predicate_bytecode.try_into().unwrap();
            println!("length: {:?}", predicate_bytes.len());

            // Add predicate coin to inputs and call contract
            let handler = contract_instance.methods().get_input_predicate(2, predicate_bytes.clone());
            let mut executable = handler.get_executable_call().await.unwrap();

            executable.tx.inputs_mut().push(predicate_coin);

            let receipts = executable
                .execute(&wallet.get_provider().unwrap())
                .await
                .unwrap();

            println!("receipts: {:#?}", receipts);
            assert_eq!(receipts[1].val().unwrap(), 1);
        }

        mod message {
            use super::*;

            #[tokio::test]
            async fn can_get_input_message_msg_id() -> Result<(), Error> {
                let (contract_instance, _, wallet, _) = get_contracts().await;

                let handler = contract_instance.methods().get_input_message_msg_id(2);
                let mut executable = handler.get_executable_call().await.unwrap();
                add_message_input(&mut executable, wallet.clone()).await;

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();

                let messages = wallet.get_messages().await?;
                let message_id: [u8; 32] = *messages[0].message_id();
                

                assert_eq!(receipts[1].data().unwrap(), message_id);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_sender() -> Result<(), Error> {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_sender(2);
                let mut executable = handler.get_executable_call().await.unwrap();
                add_message_input(&mut executable, wallet.clone()).await; // let result = contract_instance

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;

                assert_eq!(receipts[1].data().unwrap(), *messages[0].sender.hash());
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_recipient() -> Result<(), Error> {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_recipient(2);
                let mut executable = handler.get_executable_call().await.unwrap();
                add_message_input(&mut executable, wallet.clone()).await;

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                assert_eq!(receipts[1].data().unwrap(), *messages[0].recipient.hash());
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_nonce() -> Result<(), Error> {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_nonce(2);
                let mut executable = handler.get_executable_call().await.unwrap();
                add_message_input(&mut executable, wallet.clone()).await;

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();
                let messages = wallet.get_messages().await?;
                let nonce: u64 = messages[0].nonce.clone().into();
                assert_eq!(receipts[1].val().unwrap(), nonce);
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_witness_index() {
                let (contract_instance, _, _, _) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_witness_index(1)
                    .call()
                    .await
                    .unwrap();

                assert_eq!(result.value, 0);
            }

            #[tokio::test]
            async fn can_get_input_message_data_length() {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_data_length(2);
                let mut executable = handler.get_executable_call().await.unwrap();
                add_message_input(&mut executable, wallet.clone()).await;

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();

                assert_eq!(receipts[1].val().unwrap(), 3);
            }

            #[tokio::test]
            #[ignore]
            async fn can_get_input_message_predicate_length() {
                let (contract_instance, _, _, _) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_predicate_length(1)
                    .append_variable_outputs(2)
                    .append_message_outputs(2)
                    .call()
                    .await
                    .unwrap();
                // flakey. test with non-zero val
                assert_eq!(result.value, 0u16);
            }

            #[tokio::test]
            #[ignore]
            async fn can_get_input_message_predicate_data_length() {
                let (contract_instance, _, _, _) = get_contracts().await;
                let result = contract_instance
                    .methods()
                    .get_input_predicate_data_length(1)
                    .call()
                    .await
                    .unwrap();

                // flakey. test with non-zero val
                assert_eq!(result.value, 0u16);
            }

            #[tokio::test]
            async fn can_get_input_message_data() {
                let (contract_instance, _, wallet, _) = get_contracts().await;

                let handler =
                    contract_instance
                        .methods()
                        .get_input_message_data(2, 0, MESSAGE_DATA);
                let mut executable = handler.get_executable_call().await.unwrap();
                add_message_input(&mut executable, wallet.clone()).await;

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();

                println!("rec: {:#?}", receipts);
                // assert_eq!(receipts[1].val().unwrap(), 1);
            }

            #[tokio::test]
            // #[ignore]
            async fn can_get_input_message_predicate() {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let (predicate_bytecode, _, predicate_message) =
                    generate_predicate_inputs(100, vec![], &wallet).await;
                let handler = contract_instance.methods().get_input_predicate(2, predicate_bytecode.clone());
                let mut executable = handler.get_executable_call().await.unwrap();

                executable.tx.inputs_mut().push(predicate_message);

                let receipts = executable
                    .execute(&wallet.get_provider().unwrap())
                    .await
                    .unwrap();

                assert_eq!(receipts[1].data().unwrap(), predicate_bytecode);
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
            let (contract_instance, _, _, _) = get_contracts().await;
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
                let (contract_instance, _, _, _) = get_contracts().await;
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
