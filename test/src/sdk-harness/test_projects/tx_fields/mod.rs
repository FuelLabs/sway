use fuel_core_client::client::types::primitives::ChainId;
use fuel_vm::fuel_crypto::Hasher;
use fuel_vm::fuel_tx::{
    field::*, Bytes32, ConsensusParameters, ContractId, Input as TxInput, TxPointer, UtxoId,
};
use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked, Account},
    prelude::*,
    types::Bits256,
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

    let provider = setup_test_provider(coins.clone(), vec![messages], None, None).await;

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
) -> (Vec<u8>, TxInput, TxInput) {
    let predicate =
        Predicate::load_from("test_projects/tx_fields/out/debug/tx_predicate.bin").unwrap();

    let predicate_code =
        std::fs::read("test_projects/tx_fields/out/debug/tx_predicate.bin").unwrap();

    let predicate_root = predicate.address();

    let provider = wallet.provider().unwrap();
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

    let predicate_coin = &provider
        .get_coins(&predicate_root, AssetId::default())
        .await
        .unwrap()[0];
    let predicate_coin = TxInput::coin_predicate(
        UtxoId::from(predicate_coin.utxo_id.clone()),
        Address::from(predicate_coin.owner.clone()),
        predicate_coin.amount.clone().into(),
        AssetId::from(predicate_coin.asset_id.clone()),
        TxPointer::default(),
        0u32.into(),
        0,
        predicate_code.clone(),
        vec![],
    );

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

    (predicate_code, predicate_coin, predicate_message)
}

async fn add_message_input(tx: &mut ScriptTransaction, wallet: WalletUnlocked) {
    let message = &wallet.get_messages().await.unwrap()[0];

    let message_input = TxInput::message_data_signed(
        message.sender.clone().into(),
        message.recipient.clone().into(),
        message.amount,
        message.nonce,
        0,
        message.data.clone(),
    );

    tx.tx.inputs_mut().push(message_input);
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

    let mut wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None).await;
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
    use fuel_vm::prelude::Cacheable;

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

        let handler = contract_instance.methods().get_tx_inputs_count();
        let mut tx = handler.build_tx().await.unwrap();

        add_message_input(&mut tx, wallet.clone()).await;
        tx.tx.precompute(&ChainId::default()).unwrap();

        let inputs = tx.inputs().clone();

        let provider = wallet.provider().unwrap();
        let tx_id = provider.send_transaction(tx).await.unwrap();
        let receipts = provider
            .tx_status(&tx_id)
            .await
            .unwrap()
            .take_receipts_checked(None)
            .unwrap();

        assert_eq!(inputs.len() as u64, 3u64);
        assert_eq!(receipts[1].val().unwrap(), inputs.len() as u64);
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
        // let (contract_instance, _, wallet, deployment_wallet) = get_contracts().await;

        // let handler = contract_instance.methods().get_tx_witness_pointer(1);
        // let mut tx = handler.build_tx().await.unwrap();
        // deployment_wallet.sign_transaction(&mut tx).unwrap();

        // let provider = wallet.provider().unwrap();
        // let tx_id = provider.send_transaction(tx).await.unwrap();
        // let receipts = provider
        //     .tx_status(&tx_id)
        //     .await
        //     .unwrap()
        //     .take_receipts_checked(None)
        //     .unwrap();

        // assert_eq!(receipts[1].val().unwrap(), 11040);
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

        let params = wallet.provider().unwrap().consensus_parameters();
        let tx_id = tx.id(params.chain_id);

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
        use fuel_vm::{checked_transaction::EstimatePredicates, prelude::Cacheable};

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
            let (predicate_bytecode, predicate_coin, _) =
                generate_predicate_inputs(100, &wallet).await;
            let predicate_bytes: Vec<u8> = predicate_bytecode.try_into().unwrap();

            // Add predicate coin to inputs and call contract
            let handler = contract_instance
                .methods()
                .get_input_predicate(2, predicate_bytes.clone());
            let mut tx = handler.build_tx().await.unwrap();

            tx.tx.inputs_mut().push(predicate_coin);
            tx.tx.precompute(&ChainId::default()).unwrap();

            let provider = wallet.provider().unwrap();
            tx.tx
                .estimate_predicates(
                    &wallet.provider().unwrap().consensus_parameters(),
                    &provider.network_info().await.unwrap().gas_costs,
                )
                .unwrap();

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
            use fuel_vm::{checked_transaction::EstimatePredicates, prelude::Cacheable};

            use super::*;

            #[tokio::test]
            async fn can_get_input_message_sender() -> Result<()> {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_sender(2);
                let mut tx = handler.build_tx().await.unwrap();
                add_message_input(&mut tx, wallet.clone()).await; // let result = contract_instance
                tx.tx.precompute(&ChainId::default())?;

                let messages = wallet.get_messages().await?;

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data().unwrap(), *messages[0].sender.hash());
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_recipient() -> Result<()> {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_recipient(2);
                let mut tx = handler.build_tx().await.unwrap();
                add_message_input(&mut tx, wallet.clone()).await;
                tx.tx.precompute(&ChainId::default())?;

                let messages = wallet.get_messages().await?;

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(receipts[1].data().unwrap(), *messages[0].recipient.hash());
                Ok(())
            }

            #[tokio::test]
            async fn can_get_input_message_nonce() -> Result<()> {
                let (contract_instance, _, wallet, _) = get_contracts().await;
                let handler = contract_instance.methods().get_input_message_nonce(2);
                let mut tx = handler.build_tx().await.unwrap();
                add_message_input(&mut tx, wallet.clone()).await;
                tx.tx.precompute(&ChainId::default())?;

                let messages = wallet.get_messages().await?;

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let receipts = provider
                    .tx_status(&tx_id)
                    .await
                    .unwrap()
                    .take_receipts_checked(None)
                    .unwrap();

                let nonce = *messages[0].nonce.clone();
                let val = receipts[1].data().unwrap();
                assert_eq!(val, &nonce);
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
                let mut tx = handler.build_tx().await.unwrap();
                add_message_input(&mut tx, wallet.clone()).await;
                tx.tx.precompute(&ChainId::default()).unwrap();

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
                let (predicate_bytecode, _, predicate_message) =
                    generate_predicate_inputs(100, &wallet).await;
                let handler = contract_instance.methods().get_input_predicate_length(2);
                let mut tx = handler.build_tx().await.unwrap();
                tx.tx.inputs_mut().push(predicate_message);
                tx.tx.precompute(&ChainId::default()).unwrap();

                let provider = wallet.provider().unwrap();
                tx.tx
                    .estimate_predicates(
                        &wallet.provider().unwrap().consensus_parameters(),
                        &provider.network_info().await.unwrap().gas_costs,
                    )
                    .unwrap();

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
                let (_, _, predicate_message) = generate_predicate_inputs(100, &wallet).await;
                let handler = contract_instance
                    .methods()
                    .get_input_predicate_data_length(1);
                let mut tx = handler.build_tx().await.unwrap();
                tx.tx.inputs_mut().push(predicate_message);
                tx.tx.precompute(&ChainId::default()).unwrap();

                let provider = wallet.provider().unwrap();
                tx.tx
                    .estimate_predicates(
                        &wallet.provider().unwrap().consensus_parameters(),
                        &provider.network_info().await.unwrap().gas_costs,
                    )
                    .unwrap();

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

                let handler =
                    contract_instance
                        .methods()
                        .get_input_message_data(2, 0, MESSAGE_DATA);
                let mut tx = handler.build_tx().await.unwrap();
                add_message_input(&mut tx, wallet.clone()).await;
                tx.tx.precompute(&ChainId::default()).unwrap();

                let provider = wallet.provider().unwrap();
                let tx_id = provider.send_transaction(tx).await.unwrap();
                let tx = provider
                    .get_transaction_by_id(&tx_id)
                    .await
                    .unwrap()
                    .unwrap();
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
                let (predicate_bytecode, _, predicate_message) =
                    generate_predicate_inputs(100, &wallet).await;
                let predicate_bytes: Vec<u8> = predicate_bytecode.try_into().unwrap();
                let handler = contract_instance
                    .methods()
                    .get_input_predicate(2, predicate_bytes.clone());

                let mut tx = handler.build_tx().await.unwrap();

                tx.tx.inputs_mut().push(predicate_message);
                tx.tx.precompute(&ChainId::default()).unwrap();
                tx.tx
                    .estimate_predicates(
                        &wallet.provider().unwrap().consensus_parameters(),
                        &wallet
                            .provider()
                            .unwrap()
                            .network_info()
                            .await
                            .unwrap()
                            .gas_costs,
                    )
                    .unwrap();

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
