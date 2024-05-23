use fuel_vm::fuel_crypto::Hasher;
use fuel_vm::fuel_tx::{Bytes32, ConsensusParameters, ContractId, Input as TxInput};
use fuels::types::transaction_builders::TransactionBuilder;
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked, Account},
    prelude::*,
    types::{input::Input as SdkInput, Bits256},
};

const MESSAGE_DATA: [u8; 3] = [1u8, 2u8, 3u8];

abigen!(
    Contract(
        name = "TxContractTest",
        abi = "test_artifacts/tx_contract/out/release/tx_contract-abi.json",
    ),
    Predicate(
        name = "TestPredicate",
        abi = "test_projects/tx_fields/out/release/tx_fields-abi.json"
    ),
    Predicate(
        name = "TestOutputPredicate",
        abi = "test_artifacts/tx_output_predicate/out/release/tx_output_predicate-abi.json"
    )
);

async fn get_contracts(
    msg_has_data: bool,
) -> (
    TxContractTest<WalletUnlocked>,
    ContractId,
    WalletUnlocked,
    WalletUnlocked,
) {
    let mut wallet = WalletUnlocked::new_random(None);
    let mut deployment_wallet = WalletUnlocked::new_random(None);

    let mut deployment_coins = setup_single_asset_coins(
        deployment_wallet.address(),
        AssetId::BASE,
        120,
        DEFAULT_COIN_AMOUNT,
    );

    let mut coins = setup_single_asset_coins(wallet.address(), AssetId::BASE, 100, 1000);

    coins.append(&mut deployment_coins);

    let msg = setup_single_message(
        &Bech32Address {
            hrp: "".to_string(),
            hash: Default::default(),
        },
        wallet.address(),
        DEFAULT_COIN_AMOUNT,
        69.into(),
        if msg_has_data {
            MESSAGE_DATA.to_vec()
        } else {
            vec![]
        },
    );

    let provider = setup_test_provider(coins.clone(), vec![msg], None, None)
        .await
        .unwrap();

    wallet.set_provider(provider.clone());
    deployment_wallet.set_provider(provider);

    let contract_id = Contract::load_from(
        "test_artifacts/tx_contract/out/release/tx_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
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
    let predicate = Predicate::load_from("test_projects/tx_fields/out/release/tx_fields.bin")
        .unwrap()
        .with_provider(provider.clone());

    let predicate_code = predicate.code().to_vec();

    let predicate_root = predicate.address();

    let balance: u64 = wallet.get_asset_balance(&AssetId::default()).await.unwrap();

    assert!(balance >= amount);

    wallet
        .transfer(
            predicate_root,
            amount,
            AssetId::default(),
            TxPolicies::default(),
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
        predicate_address,
        message.amount,
        message.nonce,
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

    let predicate_data = TestOutputPredicateEncoder::default()
        .encode_data(0, Bits256([0u8; 32]), Bits256(*wallet1.address().hash()))
        .unwrap();

    let predicate = Predicate::load_from(
        "test_artifacts/tx_output_predicate/out/release/tx_output_predicate.bin",
    )
    .unwrap()
    .with_data(predicate_data)
    .with_provider(wallet1.try_provider().unwrap().clone());

    wallet1
        .transfer(predicate.address(), 100, asset_id1, TxPolicies::default())
        .await
        .unwrap();

    wallet1
        .transfer(predicate.address(), 100, asset_id2, TxPolicies::default())
        .await
        .unwrap();

    (wallet1, wallet2, predicate, asset_id1, asset_id2)
}

mod tx {
    use super::*;
    use fuel_vm::fuel_tx::field::Script;
    use fuels::types::{coin_type::CoinType, transaction::Transaction};

    #[tokio::test]
    async fn can_get_tx_type() {
        let (contract_instance, _, _, _) = get_contracts(true).await;

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
    async fn can_get_tip() {
        let (contract_instance, _, _, _) = get_contracts(true).await;
        let tip = 3;

        let result = contract_instance
            .methods()
            .get_tx_tip()
            .with_tx_policies(TxPolicies::default().with_tip(tip))
            .call()
            .await
            .unwrap();

        assert_eq!(result.value, tip);
    }

    #[tokio::test]
    async fn can_get_script_gas_limit() {
        let (contract_instance, _, _, _) = get_contracts(true).await;
        let script_gas_limit = 1792384;

        let result = contract_instance
            .methods()
            .get_script_gas_limit()
            .with_tx_policies(TxPolicies::default().with_script_gas_limit(script_gas_limit))
            .call()
            .await
            .unwrap();

        assert_eq!(result.value, script_gas_limit);
    }

    #[tokio::test]
    async fn can_get_maturity() {
        let (contract_instance, _, _, _) = get_contracts(true).await;
        let maturity = 0;

        let result = contract_instance
            .methods()
            .get_tx_maturity()
            .with_tx_policies(TxPolicies::default().with_maturity(maturity))
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, maturity);
    }

    #[tokio::test]
    async fn can_get_script_length() {
        let (contract_instance, _, _, _) = get_contracts(true).await;
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
        let (contract_instance, _, _, _) = get_contracts(true).await;
        // TODO make this programmatic.
        let script_data_length = 121;

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
        let (contract_instance, _, wallet, _) = get_contracts(false).await;
        let message = &wallet.get_messages().await.unwrap()[0];
        let provider = wallet.provider().unwrap();

        let mut builder = contract_instance
            .methods()
            .get_tx_inputs_count()
            .transaction_builder()
            .await
            .unwrap();

        builder.inputs_mut().push(SdkInput::ResourceSigned {
            resource: CoinType::Message(message.clone()),
        });

        builder.add_signer(wallet.clone()).unwrap();

        let tx = builder.build(provider).await.unwrap();

        let tx_inputs = tx.inputs().clone();

        let provider = wallet.provider().unwrap();
        let tx_id = provider.send_transaction(tx).await.unwrap();

        let receipts = provider
            .tx_status(&tx_id)
            .await
            .unwrap()
            .take_receipts_checked(None)
            .unwrap();

        assert_eq!(tx_inputs.len() as u64, 2u64);
        assert_eq!(
            receipts[1].data(),
            Some((tx_inputs.len() as u64).to_be_bytes().as_slice())
        );
    }

    #[tokio::test]
    async fn can_get_outputs_count() {
        let (contract_instance, _, _, _) = get_contracts(true).await;

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
        let (contract_instance, _, _, _) = get_contracts(true).await;
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
        let (contract_instance, _, _, _) = get_contracts(true).await;

        let response = contract_instance
            .methods()
            .get_tx_witness_pointer(0)
            .call()
            .await
            .unwrap();

        assert_eq!(response.value, 11024);
    }

    #[tokio::test]
    async fn can_get_witness_data_length() {
        let (contract_instance, _, _, _) = get_contracts(true).await;

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
        let (contract_instance, _, wallet, _) = get_contracts(true).await;

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
    async fn can_get_script_start_offset() {
        let (contract_instance, _, _, _) = get_contracts(true).await;

        let script_start_offset = ConsensusParameters::default().tx_params().tx_offset()
            + fuel_vm::fuel_tx::Script::script_offset_static();

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
        let (contract_instance, _, _, _) = get_contracts(true).await;

        let tx = contract_instance
            .methods()
            .get_tx_script_bytecode_hash()
            .build_tx()
            .await
            .unwrap();

        let script = tx.script();
        let hash = if script.len() > 1 {
            Hasher::hash(script)
        } else {
            Hasher::hash(vec![])
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
        let (contract_instance, _, wallet, _) = get_contracts(true).await;

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
        let (contract_instance, _, _, _) = get_contracts(true).await;
        let result = contract_instance
            .methods()
            .get_tx_script_data_start_pointer()
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, 10392)
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
                let (contract_instance, _, _, _) = get_contracts(true).await;
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
            let (contract_instance, _, _, _) = get_contracts(true).await;

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
            let (contract_instance, _, _, _) = get_contracts(true).await;
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
            let (contract_instance, _, _, deployment_wallet) = get_contracts(true).await;

            let owner_result = contract_instance
                .methods()
                .get_input_coin_owner(1)
                .call()
                .await
                .unwrap();

            assert_eq!(owner_result.value, deployment_wallet.address().into());
        }

        #[tokio::test]
        async fn can_get_input_coin_predicate() {
            let (contract_instance, _, wallet, _) = get_contracts(true).await;
            let (predicate_bytes, predicate_coin, _) =
                generate_predicate_inputs(100, &wallet).await;
            let provider = wallet.provider().unwrap();

            // Add predicate coin to inputs and call contract
            let mut tb = contract_instance
                .methods()
                .get_input_predicate(1, predicate_bytes.clone())
                .transaction_builder()
                .await
                .unwrap();

            tb.inputs_mut().push(predicate_coin);

            let tx = tb.build(provider).await.unwrap();

            let provider = wallet.provider().unwrap();

            let tx_id = provider.send_transaction(tx).await.unwrap();
            let receipts = provider
                .tx_status(&tx_id)
                .await
                .unwrap()
                .take_receipts_checked(None)
                .unwrap();
            assert_eq!(receipts[1].data(), Some(&[1u8][..]));
        }

        mod message {
            use fuels::types::{coin_type::CoinType, transaction_builders::TransactionBuilder};

            use super::*;

            #[tokio::test]
            async fn can_get_input_message_sender() {
                let (contract_instance, _, wallet, _) = get_contracts(false).await;
                let provider = wallet.provider().unwrap();

                let message = &wallet.get_messages().await.unwrap()[0];
                let mut builder = contract_instance
                    .methods()
                    .get_input_message_sender(1)
                    .transaction_builder()
                    .await
                    .unwrap();

                builder.inputs_mut().push(SdkInput::ResourceSigned {
                    resource: CoinType::Message(message.clone()),
                });

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();

                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data().unwrap(), *message.sender.hash());
            }

            #[tokio::test]
            async fn can_get_input_message_recipient() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let provider = wallet.provider().unwrap();

                let message = &wallet.get_messages().await.unwrap()[0];
                let recipient = message.recipient.hash;
                let mut builder = contract_instance
                    .methods()
                    .get_input_message_recipient(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(SdkInput::ResourceSigned {
                    resource: CoinType::Message(message.clone()),
                });

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

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
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let provider = wallet.provider().unwrap();

                let message = &wallet.get_messages().await.unwrap()[0];
                let nonce = message.nonce;

                let mut builder = contract_instance
                    .methods()
                    .get_input_message_nonce(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(SdkInput::ResourceSigned {
                    resource: CoinType::Message(message.clone()),
                });

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

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
                let (contract_instance, _, _, _) = get_contracts(true).await;
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
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let provider = wallet.provider().unwrap();

                let message = &wallet.get_messages().await.unwrap()[0];
                let mut builder = contract_instance
                    .methods()
                    .get_input_message_data_length(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(SdkInput::ResourceSigned {
                    resource: CoinType::Message(message.clone()),
                });

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data(), Some(&[0, 3][..]));
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_length() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let (predicate_bytecode, message, _) =
                    generate_predicate_inputs(100, &wallet).await;
                let provider = wallet.provider().unwrap();

                let mut builder = contract_instance
                    .methods()
                    .get_input_predicate_length(3)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(message);

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

                let provider = wallet.provider().unwrap();

                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                let len = predicate_bytecode.len() as u16;
                assert_eq!(receipts[1].data(), Some(len.to_be_bytes().as_slice()));
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_data_length() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let (_, message, _) = generate_predicate_inputs(100, &wallet).await;
                let provider = wallet.provider().unwrap();

                let mut builder = contract_instance
                    .methods()
                    .get_input_predicate_data_length(1)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(message);

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

                let provider = wallet.provider().unwrap();

                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data(), Some(0u16.to_le_bytes().as_slice()));
            }

            #[tokio::test]
            async fn can_get_input_message_data() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let message = &wallet.get_messages().await.unwrap()[0];
                let provider = wallet.provider().unwrap();

                let mut builder = contract_instance
                    .methods()
                    .get_input_message_data(3, 0, MESSAGE_DATA)
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(SdkInput::ResourceSigned {
                    resource: CoinType::Message(message.clone()),
                });

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();

                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data(), Some(&[1][..]));
            }

            #[tokio::test]
            async fn can_get_input_message_predicate() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let (predicate_bytecode, message, _) =
                    generate_predicate_inputs(100, &wallet).await;
                let provider = wallet.provider().unwrap();

                let handler = contract_instance
                    .methods()
                    .get_input_predicate(3, predicate_bytecode);

                let mut builder = handler.transaction_builder().await.unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();

                builder.inputs_mut().push(message);

                builder.add_signer(wallet.clone()).unwrap();

                let tx = builder.build(provider).await.unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data(), Some(1u8.to_le_bytes().as_slice()));
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
            let (contract_instance, _, _, _) = get_contracts(true).await;
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
                    TxPolicies::default(),
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
                let (contract_instance, _, _, _) = get_contracts(true).await;
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
                    TxPolicies::default(),
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
                    TxPolicies::default(),
                )
                .await
                .unwrap();
        }
    }
}
