use fuel_vm::fuel_crypto::Hasher;
use fuel_vm::fuel_tx::{Bytes32, ConsensusParameters, ContractId, Input as TxInput};
use fuels::types::transaction_builders::TransactionBuilder;
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked, Account},
    prelude::*,
    types::{input::Input as SdkInput, Bits256},
};

use std::str::FromStr;

const MESSAGE_DATA: [u8; 3] = [1u8, 2u8, 3u8];

abigen!(
    Contract(
        name = "TxContractTest",
        abi = "test_artifacts/tx_contract/out/debug/tx_contract-abi.json",
    ),
    Predicate(
        name = "TestPredicate",
        abi = "test_projects/tx_fields/out/debug/tx_predicate-abi.json"
    ),
    Predicate(
        name = "TestOutputPredicate",
        abi = "test_artifacts/tx_output_predicate/out/debug/tx_output_predicate-abi.json"
    )
);

async fn get_contracts() -> (
    TxContractTest<WalletUnlocked>,
    ContractId,
    WalletUnlocked,
    WalletUnlocked,
) {
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
        69.into(),
        MESSAGE_DATA.to_vec(),
    );

    let provider = setup_test_provider(coins.clone(), vec![messages], None, None)
        .await
        .unwrap();

    wallet.set_provider(provider.clone());
    deployment_wallet.set_provider(provider);

    let contract_id = Contract::load_from(
        "test_artifacts/tx_contract/out/debug/tx_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxParameters::default())
    .await
    .unwrap();

    let instance = TxContractTest::new(contract_id.clone(), deployment_wallet.clone());

    (instance, contract_id.into(), wallet, deployment_wallet)
}

async fn generate_predicate_inputs(
    amount: u64,
    wallet: &WalletUnlocked,
) -> (Vec<u8>, SdkInput, TxInput) {
    let provider = wallet.provider().unwrap();
    let predicate = Predicate::load_from("test_projects/tx_fields/out/debug/tx_predicate.bin")
        .unwrap()
        .with_provider(provider.clone());

    let predicate_code = predicate.code().clone();

    let predicate_root = predicate.address();

    let balance: u64 = wallet.get_asset_balance(&AssetId::default()).await.unwrap();

    assert!(balance >= amount);

    wallet
        .transfer(
            predicate_root,
            amount,
            AssetId::default(),
            TxParameters::default(),
        )
        .await
        .unwrap();

    let predicate_input = predicate
        .get_asset_inputs_for_amount(AssetId::default(), amount)
        .await
        .unwrap()
        .first()
        .unwrap()
        .to_owned();

    let message = &wallet.get_messages().await.unwrap()[0];
    let predicate_address: Address = predicate.address().into();

    let predicate_message = TxInput::message_coin_predicate(
        message.sender.clone().into(),
        predicate_address.clone().into(),
        message.amount,
        message.nonce.clone(),
        0,
        predicate_code.clone(),
        vec![],
    );

    (predicate_code, predicate_input, predicate_message)
}

async fn setup_output_predicate() -> (WalletUnlocked, WalletUnlocked, Predicate, AssetId, AssetId) {
    let asset_id1 = AssetId::default();
    let asset_id2 = AssetId::new([2u8; 32]);
    let wallets_config = WalletsConfig::new_multiple_assets(
        2,
        vec![
            AssetConfig {
                id: asset_id1,
                num_coins: 1,
                coin_amount: 1_000,
            },
            AssetConfig {
                id: asset_id2,
                num_coins: 1,
                coin_amount: 1_000,
            },
        ],
    );

    let mut wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None)
        .await
        .unwrap();
    let wallet1 = wallets.pop().unwrap();
    let wallet2 = wallets.pop().unwrap();

    let predicate_data = TestOutputPredicateEncoder::encode_data(
        0,
        Bits256([0u8; 32]),
        Bits256(*wallet1.address().hash()),
    );

    let predicate = Predicate::load_from(
        "test_artifacts/tx_output_predicate/out/debug/tx_output_predicate.bin",
    )
    .unwrap()
    .with_data(predicate_data)
    .with_provider(wallet1.try_provider().unwrap().clone());

    wallet1
        .transfer(predicate.address(), 100, asset_id1, TxParameters::default())
        .await
        .unwrap();

    wallet1
        .transfer(predicate.address(), 100, asset_id2, TxParameters::default())
        .await
        .unwrap();

    (wallet1, wallet2, predicate, asset_id1, asset_id2)
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
        assert_eq!(result.value, super::Transaction::Script);
    }

    #[tokio::test]
    async fn can_get_gas_price() {
        let (contract_instance, _, _, _) = get_contracts().await;
        let gas_price = 3;

        let result = contract_instance
            .methods()
            .get_tx_gas_price()
            .tx_params(TxParameters::default().with_gas_price(gas_price))
            .call()
            .await
            .unwrap();

        assert_eq!(result.value, gas_price);
    }

    #[tokio::test]
    async fn can_get_gas_limit() {
        let (contract_instance, _, _, _) = get_contracts().await;
        let gas_limit = 1792384;

        let result = contract_instance
            .methods()
            .get_tx_gas_limit()
            .tx_params(TxParameters::default().with_gas_limit(gas_limit))
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
        let script_length = 24;

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
        let script_data_length = 80;

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
        let message = &wallet.get_messages().await.unwrap()[0];

        let mut builder = contract_instance.methods()
            .get_tx_inputs_count()
            .transaction_builder()
            .await
            .unwrap();
        
        wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

        builder.inputs_mut().push(fuels::types::input::Input::ResourceSigned { 
            resource: fuels::types::coin_type::CoinType::Message(message.clone()) 
        });
        
        wallet.sign_transaction(&mut builder);

        let tx = builder.build()
            .unwrap();

        let tx_inputs = tx.inputs().clone();

        let provider = wallet.provider().unwrap();
        let tx_id = provider.send_transaction(tx).await.unwrap();

        let receipts = provider
            .tx_status(&tx_id)
            .await
            .unwrap()
            .take_receipts_checked(None)
            .unwrap();

        assert_eq!(tx_inputs.len() as u64, 4u64);
        assert_eq!(receipts[1].val().unwrap(), tx_inputs.len() as u64);
    }

    #[tokio::test]
    async fn can_get_outputs_count() {
        let (contract_instance, _, _, _) = get_contracts().await;

        let call_handler = contract_instance.methods().get_tx_outputs_count();
        let tx = call_handler.build_tx().await.unwrap();
        let outputs = tx.outputs();

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
        let (contract_instance, _, wallet, deployment_wallet) = get_contracts().await;

        let mut builder = contract_instance.methods()
            .get_tx_witness_pointer(1)
            .transaction_builder()
            .await
            .unwrap();
        deployment_wallet.sign_transaction(&mut builder);

        wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();
        wallet.sign_transaction(&mut builder);

        let tx = builder.build().unwrap();

        let provider = wallet.provider().unwrap();
        let tx_id = provider.send_transaction(tx).await.unwrap();
        let receipts = provider
            .tx_status(&tx_id)
            .await
            .unwrap()
            .take_receipts_checked(None)
            .unwrap();

        assert_eq!(receipts[1].val().unwrap(), 11200);
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
        let tx = handler.build_tx().await.unwrap();
        let witnesses = tx.witnesses().clone();

        let provider = wallet.provider().unwrap();
        let tx_id = provider.send_transaction(tx).await.unwrap();
        let receipts = provider
            .tx_status(&tx_id)
            .await
            .unwrap()
            .take_receipts_checked(None)
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
            .build_tx()
            .await
            .unwrap();

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
        let tx = handler.build_tx().await.unwrap();

        let provider = wallet.provider().unwrap();
        let tx_id = provider.send_transaction(tx).await.unwrap();
        let receipts = provider
            .tx_status(&tx_id)
            .await
            .unwrap()
            .take_receipts_checked(None)
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
        assert_eq!(result.value, 10368)
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
                    .unwrap()
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
            assert_eq!(result.value, Input::Contract);

            let result = contract_instance
                .methods()
                .get_input_type(1)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, Input::Coin);
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
            let (predicate_bytes, predicate_coin, _) =
                generate_predicate_inputs(100, &wallet).await;

            // Add predicate coin to inputs and call contract
            let mut tb = contract_instance
                .methods()
                .get_input_predicate(1, predicate_bytes.clone())
                .transaction_builder()
                .await
                .unwrap();

            tb.inputs_mut().push(predicate_coin);

            let tx = tb.build().unwrap();

            let provider = wallet.provider().unwrap();

            let tx_id = provider.send_transaction(tx).await.unwrap();
            let receipts = provider
                .tx_status(&tx_id)
                .await
                .unwrap()
                .take_receipts_checked(None)
                .unwrap();

            assert_eq!(receipts[1].val().unwrap(), 1);
        }

        mod message {
            use fuels::types::transaction_builders::TransactionBuilder;

            use super::*;

            #[tokio::test]
            async fn can_get_input_message_sender() {
                let (contract_instance, _, wallet, _) = get_contracts().await;

                let message = &wallet.get_messages().await.unwrap()[0];
                let mut builder = contract_instance.methods()
                    .get_input_message_sender(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();
                
                builder.inputs_mut().push(fuels::types::input::Input::ResourceSigned { 
                    resource: fuels::types::coin_type::CoinType::Message(message.clone())
                });
                
                
                wallet.sign_transaction(&mut builder);

                let tx = builder.build().unwrap();
                dbg!(tx.inputs());
                let messages = wallet.get_messages().await.unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();

                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data().unwrap(), *messages[0].sender.hash());
            }

            #[tokio::test]
            async fn can_get_input_message_recipient() {
                let (contract_instance, _, wallet, _) = get_contracts().await;

                let message = &wallet.get_messages().await.unwrap()[0];
                let recipient = message.recipient.hash.clone();
                let mut builder = contract_instance.methods()
                    .get_input_message_recipient(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();
                
                builder.inputs_mut().push(fuels::types::input::Input::ResourceSigned { 
                    resource: fuels::types::coin_type::CoinType::Message(message.clone()) 
                });

                wallet.sign_transaction(&mut builder);
                
                let tx = builder.build().unwrap();
                dbg!(tx.inputs());
                
                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();
            
                assert_eq!(receipts[1].data().unwrap(), recipient.as_slice());
            }

            #[tokio::test]
            async fn can_get_input_message_nonce() {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                
                let message = &wallet.get_messages().await.unwrap()[0];
                let nonce = message.nonce.clone();

                let mut builder = contract_instance.methods()
                    .get_input_message_nonce(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(fuels::types::input::Input::ResourceSigned { 
                    resource: fuels::types::coin_type::CoinType::Message(message.clone()) 
                });

                wallet.sign_transaction(&mut builder);

                let tx = builder.build().unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data().unwrap(), nonce.as_slice());
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

                let message = &wallet.get_messages().await.unwrap()[0];
                let mut builder = contract_instance.methods()
                    .get_input_message_data_length(3)
                    .transaction_builder()
                    .await
                    .unwrap();
                   
                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(fuels::types::input::Input::ResourceSigned { 
                    resource: fuels::types::coin_type::CoinType::Message(message.clone()) 
                });

                wallet.sign_transaction(&mut builder);
                    
                let tx = builder.build()
                    .unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].val().unwrap(), 3);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_length() {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let (predicate_bytecode, message, _) =
                    generate_predicate_inputs(100, &wallet).await;

                let mut builder = contract_instance.methods()
                    .get_input_predicate_length(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(message);
                
                wallet.sign_transaction(&mut builder);

                let tx = builder.build().unwrap();

                let provider = wallet.provider().unwrap();

                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].val().unwrap(), predicate_bytecode.len() as u64);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_data_length() {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let (_, message, _) = generate_predicate_inputs(100, &wallet).await;

                let mut builder = contract_instance
                    .methods()
                    .get_input_predicate_data_length(1)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(message);
                
                wallet.sign_transaction(&mut builder);

                let tx = builder.build().unwrap();

                let provider = wallet.provider().unwrap();

                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].val().unwrap(), 0);
            }

            #[tokio::test]
            async fn can_get_input_message_data() {
                let (contract_instance, _, wallet, _) = get_contracts().await;

                let builder =  contract_instance
                    .methods()
                    .get_input_message_data(2, 0, MESSAGE_DATA)
                    .transaction_builder()
                    .await
                    .unwrap();
                let tx = builder.build().unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();

                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].val().unwrap(), 1);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate() {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let (predicate_bytecode, message, _) =
                    generate_predicate_inputs(100, &wallet).await;
                let predicate_bytes: Vec<u8> = predicate_bytecode.try_into().unwrap();
                let handler = contract_instance
                    .methods()
                    .get_input_predicate(3, predicate_bytes.clone());

                let mut builder = handler
                    .transaction_builder()
                    .await
                    .unwrap();
                
                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(message);
                
                wallet.sign_transaction(&mut builder);

                let tx = builder.build().unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].val().unwrap(), 1);
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
            assert_eq!(result.value, Output::Contract);
        }

        #[tokio::test]
        async fn can_get_tx_output_details() {
            let (wallet, _, predicate, asset_id, _) = setup_output_predicate().await;

            let balance = predicate.get_asset_balance(&asset_id).await.unwrap();

            let transfer_amount = 10;
            predicate
                .transfer(
                    wallet.address(),
                    transfer_amount,
                    asset_id,
                    TxParameters::default(),
                )
                .await
                .unwrap();

            let new_balance = predicate.get_asset_balance(&asset_id).await.unwrap();

            assert!(balance - transfer_amount == new_balance);
        }
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

        #[tokio::test]
        #[should_panic]
        async fn fails_output_predicate_when_incorrect_asset() {
            let (wallet1, _, predicate, _, asset_id2) = setup_output_predicate().await;

            let transfer_amount = 10;
            predicate
                .transfer(
                    wallet1.address(),
                    transfer_amount,
                    asset_id2,
                    TxParameters::default(),
                )
                .await
                .unwrap();
        }

        #[tokio::test]
        #[should_panic]
        async fn fails_output_predicate_when_incorrect_to() {
            let (_, wallet2, predicate, asset_id1, _) = setup_output_predicate().await;

            let transfer_amount = 10;
            predicate
                .transfer(
                    wallet2.address(),
                    transfer_amount,
                    asset_id1,
                    TxParameters::default(),
                )
                .await
                .unwrap();
        }
    }
}
