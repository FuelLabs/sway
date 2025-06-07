use fuel_vm::fuel_crypto::Hasher;
use fuel_vm::fuel_tx::{ContractId, Input as TxInput};
use fuels::core::codec::EncoderConfig;
use fuels::types::transaction_builders::TransactionBuilder;
use fuels::{
    accounts::{predicate::Predicate, signers::private_key::PrivateKeySigner},
    prelude::*,
    tx::StorageSlot,
    types::{input::Input as SdkInput, output::Output as SdkOutput, Bits256, ChainId},
};
use std::fs;

const MESSAGE_DATA: [u8; 3] = [1u8, 2u8, 3u8];
const TX_CONTRACT_BYTECODE_PATH: &str = "test_artifacts/tx_contract/out/release/tx_contract.bin";
const TX_OUTPUT_PREDICATE_BYTECODE_PATH: &str =
    "test_artifacts/tx_output_predicate/out/release/tx_output_predicate.bin";
const TX_OUTPUT_CONTRACT_BYTECODE_PATH: &str =
    "test_artifacts/tx_output_contract/out/release/tx_output_contract.bin";
const TX_FIELDS_PREDICATE_BYTECODE_PATH: &str = "test_projects/tx_fields/out/release/tx_fields.bin";
const TX_CONTRACT_CREATION_PREDICATE_BYTECODE_PATH: &str =
    "test_artifacts/tx_output_contract_creation_predicate/out/release/tx_output_contract_creation_predicate.bin";
const TX_TYPE_PREDICATE_BYTECODE_PATH: &str =
    "test_artifacts/tx_type_predicate/out/release/tx_type_predicate.bin";
const TX_WITNESS_PREDICATE_BYTECODE_PATH: &str =
    "test_artifacts/tx_witness_predicate/out/release/tx_witness_predicate.bin";
const TX_INPUT_COUNT_PREDICATE_BYTECODE_PATH: &str =
    "test_artifacts/tx_input_count_predicate/out/release/tx_input_count_predicate.bin";
const TX_OUTPUT_COUNT_PREDICATE_BYTECODE_PATH: &str =
    "test_artifacts/tx_output_count_predicate/out/release/tx_output_count_predicate.bin";

use crate::tx_fields::Output as SwayOutput;
use crate::tx_fields::Transaction as SwayTransaction;

abigen!(
    Contract(
        name = "TxContractTest",
        abi = "test_artifacts/tx_contract/out/release/tx_contract-abi.json",
    ),
    Contract(
        name = "TxOutputContract",
        abi = "test_artifacts/tx_output_contract/out/release/tx_output_contract-abi.json",
    ),
    Predicate(
        name = "TestPredicate",
        abi = "test_projects/tx_fields/out/release/tx_fields-abi.json"
    ),
    Predicate(
        name = "TestOutputPredicate",
        abi = "test_artifacts/tx_output_predicate/out/release/tx_output_predicate-abi.json"
    ),
    Predicate(
        name = "TestTxTypePredicate",
        abi = "test_artifacts/tx_type_predicate/out/release/tx_type_predicate-abi.json"
    ),
    Predicate(
        name = "TestTxWitnessPredicate",
        abi = "test_artifacts/tx_witness_predicate/out/release/tx_witness_predicate-abi.json"
    ),
    Predicate(
        name = "TestTxInputCountPredicate",
        abi = "test_artifacts/tx_input_count_predicate/out/release/tx_input_count_predicate-abi.json"
    ),
    Predicate(
        name = "TestTxOutputCountPredicate",
        abi = "test_artifacts/tx_output_count_predicate/out/release/tx_output_count_predicate-abi.json"
    )
);

async fn get_contracts(msg_has_data: bool) -> (TxContractTest<Wallet>, ContractId, Wallet, Wallet) {
    let wallet_signer = PrivateKeySigner::random(&mut rand::thread_rng());
    let deployment_signer = PrivateKeySigner::random(&mut rand::thread_rng());

    let mut deployment_coins = setup_single_asset_coins(
        deployment_signer.address(),
        AssetId::BASE,
        120,
        DEFAULT_COIN_AMOUNT,
    );

    let mut coins = setup_single_asset_coins(wallet_signer.address(), AssetId::BASE, 100, 1000);

    coins.append(&mut deployment_coins);

    let msg = setup_single_message(
        &Bech32Address {
            hrp: "".to_string(),
            hash: Default::default(),
        },
        wallet_signer.address(),
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
    let wallet = Wallet::new(wallet_signer, provider.clone());
    let deployment_wallet = Wallet::new(deployment_signer, provider);

    let contract_id = Contract::load_from(TX_CONTRACT_BYTECODE_PATH, LoadConfiguration::default())
        .unwrap()
        .deploy(&deployment_wallet, TxPolicies::default())
        .await
        .unwrap()
        .contract_id;

    let instance = TxContractTest::new(contract_id.clone(), wallet.clone());

    (instance, contract_id.into(), wallet, deployment_wallet)
}

async fn generate_predicate_inputs(amount: u64, wallet: &Wallet) -> (Vec<u8>, SdkInput, TxInput) {
    let provider = wallet.provider();
    let predicate = Predicate::load_from(TX_FIELDS_PREDICATE_BYTECODE_PATH)
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
        .get_asset_inputs_for_amount(AssetId::default(), amount.into(), None)
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

async fn setup_output_predicate(
    index: u64,
    expected_output_type: SwayOutput,
) -> (Wallet, Wallet, Predicate, AssetId, AssetId) {
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
        .encode_data(
            index,
            Bits256([0u8; 32]),
            Bits256(*wallet1.address().hash()),
            expected_output_type,
        )
        .unwrap();

    let predicate = Predicate::load_from(TX_OUTPUT_PREDICATE_BYTECODE_PATH)
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

        assert_eq!(result.value, Some(tip));

        let no_tip = contract_instance
            .methods()
            .get_tx_tip()
            .call()
            .await
            .unwrap();

        assert_eq!(no_tip.value, None);
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
        assert_eq!(result.value, Some(maturity as u32));

        // Assert none is returned with no maturity
        let no_maturity = contract_instance
            .methods()
            .get_tx_maturity()
            .call()
            .await
            .unwrap();
        assert_eq!(no_maturity.value, None);
    }

    #[tokio::test]
    async fn can_get_expiration() {
        let (contract_instance, _, wallet, _) = get_contracts(true).await;

        let provider = wallet.try_provider().unwrap();

        // This should be an error because we are not at the genesis block
        let err = contract_instance
            .methods()
            .get_tx_expiration()
            .with_tx_policies(TxPolicies::default().with_expiration(0))
            .call()
            .await
            .expect_err("expiration reached");

        assert!(err.to_string().contains("TransactionExpiration"));

        let result = contract_instance
            .methods()
            .get_tx_expiration()
            .with_tx_policies(TxPolicies::default().with_expiration(10))
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, Some(10 as u32));

        let result = contract_instance
            .methods()
            .get_tx_expiration()
            .with_tx_policies(TxPolicies::default().with_expiration(1234567890))
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, Some(1234567890 as u32));

        let result = contract_instance
            .methods()
            .get_tx_expiration()
            .with_tx_policies(TxPolicies::default().with_expiration(u32::MAX.into()))
            .call()
            .await
            .unwrap();
        assert_eq!(result.value, Some(u32::MAX));

        // Assert none is returned with no expiration
        let no_expiration = contract_instance
            .methods()
            .get_tx_expiration()
            .call()
            .await
            .unwrap();
        assert_eq!(no_expiration.value, None);

        // Assert tx errors after expiration
        let _ = provider.produce_blocks(15, None).await;
        let err = contract_instance
            .methods()
            .get_tx_expiration()
            .with_tx_policies(TxPolicies::default().with_expiration(10))
            .call()
            .await
            .expect_err("expiration reached");

        assert!(err.to_string().contains("TransactionExpiration"));
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
        assert_eq!(result.value, Some(script_length));
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
        assert_eq!(result.value, Some(script_data_length));
    }

    #[tokio::test]
    async fn can_get_inputs_count() {
        let (contract_instance, _, wallet, _) = get_contracts(false).await;
        let message = &wallet.get_messages().await.unwrap()[0];

        let response = contract_instance
            .methods()
            .get_tx_inputs_count()
            .with_inputs(vec![SdkInput::ResourceSigned {
                resource: CoinType::Message(message.clone()),
            }])
            .call()
            .await
            .unwrap();
        assert_eq!(response.value, 2u64);
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

        assert_eq!(result.value, outputs.len() as u16);
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
    async fn can_get_witness_data_length() {
        let (contract_instance, _, _, _) = get_contracts(true).await;

        let result = contract_instance
            .methods()
            .get_tx_witness_data_length(0)
            .call()
            .await
            .unwrap();

        assert_eq!(result.value, Some(64));
    }

    #[tokio::test]
    async fn can_get_witness_data() {
        let (contract_instance, _, wallet, _) = get_contracts(true).await;

        let handler = contract_instance.methods().get_tx_witness_data(0);
        let tx = handler.build_tx().await.unwrap();
        let witnesses = tx.witnesses().clone();

        let provider = wallet.provider();
        let tx_status = provider.send_transaction_and_await_commit(tx).await.unwrap();
        let receipts = tx_status
            .take_receipts_checked(None)
            .unwrap();

        assert_eq!(receipts[1].data().unwrap()[8..72], *witnesses[0].as_vec());
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
        assert_eq!(result.value.unwrap(), Bits256(*hash));
    }

    #[tokio::test]
    async fn can_get_tx_id() {
        let (contract_instance, _, wallet, _) = get_contracts(true).await;

        let handler = contract_instance.methods().get_tx_id();
        let tx = handler.build_tx().await.unwrap();

        let provider = wallet.provider();
        let tx_status = provider.send_transaction_and_await_commit(tx.clone()).await.unwrap();
        let receipts = tx_status
            .take_receipts_checked(None)
            .unwrap();

        let byte_array: [u8; 32] = *tx.id(ChainId::new(0));

        assert_eq!(receipts[1].data().unwrap(), byte_array);
    }

    #[tokio::test]
    async fn can_get_tx_upload() {
        // Prepare wallet and provider
        let signer = PrivateKeySigner::random(&mut rand::thread_rng());

        let num_coins = 100;
        let coins = setup_single_asset_coins(
            &signer.address(),
            AssetId::zeroed(),
            num_coins,
            DEFAULT_COIN_AMOUNT,
        );
        let provider = setup_test_provider(coins, vec![], None, None)
            .await
            .unwrap();
        let wallet = Wallet::new(signer, provider.clone());
        let consensus_params = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_params.base_asset_id();

        // Get the predicate
        let predicate_data = TestTxTypePredicateEncoder::default()
            .encode_data(SwayTransaction::Upload)
            .unwrap();
        let predicate: Predicate = Predicate::load_from(TX_TYPE_PREDICATE_BYTECODE_PATH)
            .unwrap()
            .with_provider(provider.clone())
            .with_data(predicate_data);
        let predicate_coin_amount = 100;

        // Predicate has no funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance, 0);

        // Prepare bytecode and subsections
        let bytecode = fs::read(TX_CONTRACT_BYTECODE_PATH).unwrap();
        let subsection_size = 65536;
        let subsections = UploadSubsection::split_bytecode(&bytecode, subsection_size).unwrap();

        // Transfer enough funds to the predicate for each subsection
        for _ in subsections.clone() {
            wallet
                .transfer(
                    predicate.address(),
                    predicate_coin_amount,
                    *base_asset_id,
                    TxPolicies::default(),
                )
                .await
                .unwrap();
        }

        // Predicate has funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(
            predicate_balance as usize,
            predicate_coin_amount as usize * subsections.len()
        );

        // Upload each sub section in a separate transaction and include the predicate with the transaction.
        for subsection in subsections {
            let mut builder = UploadTransactionBuilder::prepare_subsection_upload(
                subsection,
                TxPolicies::default(),
            );

            // Inputs for predicate
            let predicate_input = predicate
                .get_asset_inputs_for_amount(*base_asset_id, 1, None)
                .await
                .unwrap();

            // Outputs for predicate
            let predicate_output =
                wallet.get_asset_outputs_for_amount(&wallet.address(), *base_asset_id, 1);

            // Append the predicate to the transaction
            builder.inputs.push(predicate_input.get(0).unwrap().clone());
            builder
                .outputs
                .push(predicate_output.get(0).unwrap().clone());

            wallet.add_witnesses(&mut builder).unwrap();
            wallet.adjust_for_fee(&mut builder, 0).await.unwrap();

            // Submit the transaction
            let tx = builder.build(&provider).await.unwrap();
            provider.send_transaction_and_await_commit(tx).await.unwrap();
        }

        // The predicate has spent it's funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance, 0);
    }

    #[tokio::test]
    async fn can_get_witness_in_tx_upload() {
        // Prepare wallet and provider
        let signer = PrivateKeySigner::random(&mut rand::thread_rng());

        let num_coins = 100;
        let coins = setup_single_asset_coins(
            &signer.address(),
            AssetId::zeroed(),
            num_coins,
            DEFAULT_COIN_AMOUNT,
        );
        let provider = setup_test_provider(coins, vec![], None, None)
            .await
            .unwrap();
        let wallet = Wallet::new(signer, provider.clone());
        let consensus_params = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_params.base_asset_id();

        // Prepare bytecode and subsections
        let bytecode = fs::read(TX_CONTRACT_BYTECODE_PATH).unwrap();
        let subsection_size = 65536;
        let subsections = UploadSubsection::split_bytecode(&bytecode, subsection_size).unwrap();

        // Upload each sub section in a separate transaction and include the predicate with the transaction.
        for subsection in subsections.clone() {
            let mut builder = UploadTransactionBuilder::prepare_subsection_upload(
                subsection,
                TxPolicies::default(),
            );

            // Prepare the predicate
            let witnesses = builder.witnesses().clone();
            let predicate_data = TestTxWitnessPredicateEncoder::new(EncoderConfig {
                max_depth: 10,
                max_tokens: 100_000,
            })
            .encode_data(
                0,
                witnesses.len() as u64 + 1,
                witnesses[0].as_vec().len() as u64,
                witnesses[0].as_vec().as_slice()[0..64].try_into().unwrap(),
            )
            .unwrap();
            let predicate: Predicate = Predicate::load_from(TX_WITNESS_PREDICATE_BYTECODE_PATH)
                .unwrap()
                .with_provider(provider.clone())
                .with_data(predicate_data);
            let predicate_coin_amount = 100;

            // Predicate has no funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);
            wallet
                .transfer(
                    predicate.address(),
                    predicate_coin_amount,
                    *base_asset_id,
                    TxPolicies::default(),
                )
                .await
                .unwrap();

            // Predicate has funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(
                predicate_balance as usize,
                predicate_coin_amount as usize * subsections.len()
            );

            // Inputs for predicate
            let predicate_input = predicate
                .get_asset_inputs_for_amount(*base_asset_id, 1, None)
                .await
                .unwrap();

            // Outputs for predicate
            let predicate_output =
                wallet.get_asset_outputs_for_amount(&wallet.address(), *base_asset_id, 1);

            // Append the predicate to the transaction
            builder.inputs.push(predicate_input.get(0).unwrap().clone());
            builder
                .outputs
                .push(predicate_output.get(0).unwrap().clone());

            wallet.add_witnesses(&mut builder).unwrap();
            wallet.adjust_for_fee(&mut builder, 0).await.unwrap();

            // Submit the transaction
            let tx = builder.build(&provider).await.unwrap();
            provider.send_transaction_and_await_commit(tx).await.unwrap();

            // The predicate has spent it's funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);
        }
    }

    #[tokio::test]
    async fn can_get_tx_blob() {
        // Prepare wallet and provider
        let signer = PrivateKeySigner::random(&mut rand::thread_rng());

        let num_coins = 100;
        let coins = setup_single_asset_coins(
            signer.address(),
            AssetId::zeroed(),
            num_coins,
            DEFAULT_COIN_AMOUNT,
        );

        let provider = setup_test_provider(coins, vec![], None, None)
            .await
            .unwrap();
        let wallet = Wallet::new(signer, provider.clone());
        let consensus_params = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_params.base_asset_id();

        // Get the predicate
        let predicate_data = TestTxTypePredicateEncoder::default()
            .encode_data(SwayTransaction::Blob)
            .unwrap();
        let predicate: Predicate = Predicate::load_from(TX_TYPE_PREDICATE_BYTECODE_PATH)
            .unwrap()
            .with_provider(provider.clone())
            .with_data(predicate_data);
        let predicate_coin_amount = 100;

        // Predicate has no funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance, 0);

        // Transfer enough funds to the predicate
        wallet
            .transfer(
                predicate.address(),
                predicate_coin_amount,
                *base_asset_id,
                TxPolicies::default(),
            )
            .await
            .unwrap();

        // Predicate has funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance as usize, predicate_coin_amount as usize);

        // Prepare blobs
        let max_words_per_blob = 10_000;
        let blobs = Contract::load_from(TX_CONTRACT_BYTECODE_PATH, LoadConfiguration::default())
            .unwrap()
            .convert_to_loader(max_words_per_blob)
            .unwrap()
            .blobs()
            .to_vec();

        let blob = blobs[0].clone();
        // Inputs for predicate
        let predicate_input = predicate
            .get_asset_inputs_for_amount(*base_asset_id, 1, None)
            .await
            .unwrap();

        // Outputs for predicate
        let predicate_output =
            wallet.get_asset_outputs_for_amount(&wallet.address(), *base_asset_id, 1);

        let mut builder = BlobTransactionBuilder::default().with_blob(blob);

        // Append the predicate to the transaction
        builder.inputs.push(predicate_input.get(0).unwrap().clone());
        builder
            .outputs
            .push(predicate_output.get(0).unwrap().clone());

        wallet.adjust_for_fee(&mut builder, 0).await.unwrap();
        wallet.add_witnesses(&mut builder).unwrap();

        let tx = builder.build(&provider).await.unwrap();
        provider
            .send_transaction_and_await_commit(tx)
            .await
            .unwrap()
            .check(None)
            .unwrap();

        // The predicate has spent it's funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance, 0);
    }

    #[tokio::test]
    async fn can_get_witness_in_tx_blob() {
        // Prepare wallet and provider
        let signer = PrivateKeySigner::random(&mut rand::thread_rng());

        let num_coins = 100;
        let coins = setup_single_asset_coins(
            signer.address(),
            AssetId::zeroed(),
            num_coins,
            DEFAULT_COIN_AMOUNT,
        );

        let provider = setup_test_provider(coins, vec![], None, None)
            .await
            .unwrap();
        let wallet = Wallet::new(signer, provider.clone());
        let consensus_params = provider.consensus_parameters().await.unwrap();
        let base_asset_id = consensus_params.base_asset_id();

        // Prepare blobs
        let max_words_per_blob = 10_000;
        let blobs = Contract::load_from(TX_CONTRACT_BYTECODE_PATH, LoadConfiguration::default())
            .unwrap()
            .convert_to_loader(max_words_per_blob)
            .unwrap()
            .blobs()
            .to_vec();

        let blob = blobs[0].clone();

        let mut builder = BlobTransactionBuilder::default().with_blob(blob.clone());

        // Prepare the predicate
        let predicate_data = TestTxWitnessPredicateEncoder::new(EncoderConfig {
            max_depth: 10,
            max_tokens: 100_000,
        })
        .encode_data(
            // Blob and witnesses are just wrappers for Vec<u8>, and function the same in case of Transaction::Blob, so using blobs here instead of witnesses
            0,
            blobs.len() as u64 + 1,
            blob.len() as u64,
            blob.bytes()[0..64].try_into().unwrap(),
        )
        .unwrap();
        let predicate: Predicate = Predicate::load_from(TX_WITNESS_PREDICATE_BYTECODE_PATH)
            .unwrap()
            .with_provider(provider.clone())
            .with_data(predicate_data);
        let predicate_coin_amount = 100;

        // Predicate has no funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance, 0);
        wallet
            .transfer(
                predicate.address(),
                predicate_coin_amount,
                *base_asset_id,
                TxPolicies::default(),
            )
            .await
            .unwrap();

        // Predicate has funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance as usize, predicate_coin_amount as usize);

        // Inputs for predicate
        let predicate_input = predicate
            .get_asset_inputs_for_amount(*base_asset_id, 1, None)
            .await
            .unwrap();

        // Outputs for predicate
        let predicate_output =
            wallet.get_asset_outputs_for_amount(&wallet.address(), *base_asset_id, 1);

        // Append the predicate to the transaction
        builder.inputs.push(predicate_input.get(0).unwrap().clone());
        builder
            .outputs
            .push(predicate_output.get(0).unwrap().clone());

        wallet.add_witnesses(&mut builder).unwrap();
        wallet.adjust_for_fee(&mut builder, 0).await.unwrap();

        let tx = builder.build(provider.clone()).await.unwrap();

        provider.send_transaction_and_await_commit(tx).await.unwrap();

        // The predicate has spent it's funds
        let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
        assert_eq!(predicate_balance, 0);
    }
}

mod inputs {
    use super::*;

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
            assert_eq!(result.value, Some(Input::Contract));

            let result = contract_instance
                .methods()
                .get_input_type(1)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, Some(Input::Coin));

            // Assert invalid index returns None
            let result = contract_instance
                .methods()
                .get_input_type(100) // 100 is a very high input index
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, None);
        }

        #[tokio::test]
        async fn can_get_tx_input_amount() {
            let default_amount = 1000;
            let (contract_instance, _, _, _) = get_contracts(true).await;
            let result = contract_instance
                .methods()
                .get_input_amount(1)
                .call()
                .await
                .unwrap();

            assert_eq!(result.value, Some(default_amount));

            // Assert invalid index returns None
            let result = contract_instance
                .methods()
                .get_input_amount(0)
                .call()
                .await
                .unwrap();

            assert_eq!(result.value, None);
        }

        #[tokio::test]
        async fn can_get_tx_input_coin_owner() {
            let (contract_instance, _, wallet, _) = get_contracts(true).await;

            let owner_result = contract_instance
                .methods()
                .get_input_coin_owner(1)
                .call()
                .await
                .unwrap();

            assert_eq!(owner_result.value, Some(wallet.address().into()));

            // Assert invalid index returns None
            let result = contract_instance
                .methods()
                .get_input_coin_owner(0)
                .call()
                .await
                .unwrap();

            assert_eq!(result.value, None);
        }

        #[tokio::test]
        async fn can_get_input_coin_predicate() {
            let (contract_instance, _, wallet, _) = get_contracts(true).await;
            let (predicate_bytes, predicate_coin, _) =
                generate_predicate_inputs(100, &wallet).await;
            let provider = wallet.provider();

            // Add predicate coin to inputs and call contract
            let handler = contract_instance
                .methods()
                .get_input_predicate(1, predicate_bytes.clone());
            let mut tb = handler.transaction_builder().await.unwrap();

            tb.inputs_mut().push(predicate_coin);

            let tx = tb.enable_burn(true).build(provider).await.unwrap();

            let provider = wallet.provider();

            let tx_status = provider
                .send_transaction_and_await_commit(tx)
                .await
                .unwrap();
            let response = handler.get_response(tx_status).unwrap();

            assert!(response.value);

            // Assert invalid index returns None
            let result = contract_instance
                .methods()
                .get_input_predicate(0, predicate_bytes.clone())
                .call()
                .await
                .unwrap();

            assert_eq!(result.value, false);
        }

        #[tokio::test]
        async fn can_get_input_count_in_tx_upload() {
            // Prepare wallet and provider
            let signer = PrivateKeySigner::random(&mut rand::thread_rng());

            let num_coins = 100;
            let coins = setup_single_asset_coins(
                signer.address(),
                AssetId::zeroed(),
                num_coins,
                DEFAULT_COIN_AMOUNT,
            );
            let provider = setup_test_provider(coins, vec![], None, None)
                .await
                .unwrap();
            let wallet = Wallet::new(signer, provider.clone());
            let consensus_params = provider.consensus_parameters().await.unwrap();
            let base_asset_id = consensus_params.base_asset_id();

            // Prepare bytecode and subsections
            let bytecode = fs::read(TX_CONTRACT_BYTECODE_PATH).unwrap();
            let subsection_size = 65536;
            let subsections = UploadSubsection::split_bytecode(&bytecode, subsection_size).unwrap();

            // Upload each sub section in a separate transaction and include the predicate with the transaction.
            for subsection in subsections.clone() {
                let mut builder = UploadTransactionBuilder::prepare_subsection_upload(
                    subsection,
                    TxPolicies::default(),
                );

                // Prepare the predicate
                let predicate_data = TestTxInputCountPredicateEncoder::default()
                    .encode_data(builder.inputs().len() as u16 + 1u16) // Add one for this predicate
                    .unwrap();
                let predicate: Predicate =
                    Predicate::load_from(TX_INPUT_COUNT_PREDICATE_BYTECODE_PATH)
                        .unwrap()
                        .with_provider(provider.clone())
                        .with_data(predicate_data);
                let predicate_coin_amount = 100;

                // Predicate has no funds
                let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
                assert_eq!(predicate_balance, 0);
                wallet
                    .transfer(
                        predicate.address(),
                        predicate_coin_amount,
                        *base_asset_id,
                        TxPolicies::default(),
                    )
                    .await
                    .unwrap();

                // Predicate has funds
                let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
                assert_eq!(
                    predicate_balance as usize,
                    predicate_coin_amount as usize * subsections.len()
                );

                // Inputs for predicate
                let predicate_input = predicate
                    .get_asset_inputs_for_amount(*base_asset_id, 1, None)
                    .await
                    .unwrap();

                // Outputs for predicate
                let predicate_output =
                    wallet.get_asset_outputs_for_amount(&wallet.address(), *base_asset_id, 1);

                // Append the predicate to the transaction
                builder.inputs.push(predicate_input.get(0).unwrap().clone());
                builder
                    .outputs
                    .push(predicate_output.get(0).unwrap().clone());

                wallet.add_witnesses(&mut builder).unwrap();
                wallet.adjust_for_fee(&mut builder, 0).await.unwrap();

                // Submit the transaction
                let tx = builder.build(&provider).await.unwrap();
                provider.send_transaction_and_await_commit(tx).await.unwrap();

                // The predicate has spent it's funds
                let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
                assert_eq!(predicate_balance, 0);
            }
        }

        #[tokio::test]
        async fn can_get_input_count_in_tx_blob() {
            // Prepare wallet and provider
            let signer = PrivateKeySigner::random(&mut rand::thread_rng());

            let num_coins = 100;
            let coins = setup_single_asset_coins(
                signer.address(),
                AssetId::zeroed(),
                num_coins,
                DEFAULT_COIN_AMOUNT,
            );
            let provider = setup_test_provider(coins, vec![], None, None)
                .await
                .unwrap();
            let wallet = Wallet::new(signer, provider.clone());
            let consensus_params = provider.consensus_parameters().await.unwrap();
            let base_asset_id = consensus_params.base_asset_id();

            // Prepare blobs
            let max_words_per_blob = 10_000;
            let blobs =
                Contract::load_from(TX_CONTRACT_BYTECODE_PATH, LoadConfiguration::default())
                    .unwrap()
                    .convert_to_loader(max_words_per_blob)
                    .unwrap()
                    .blobs()
                    .to_vec();

            let blob = blobs[0].clone();

            let mut builder = BlobTransactionBuilder::default().with_blob(blob);

            // Prepare the predicate
            let predicate_data = TestTxInputCountPredicateEncoder::default()
                .encode_data(builder.inputs().len() as u16 + 1u16) // Add one for this predicate
                .unwrap();
            let predicate: Predicate = Predicate::load_from(TX_INPUT_COUNT_PREDICATE_BYTECODE_PATH)
                .unwrap()
                .with_provider(provider.clone())
                .with_data(predicate_data);
            let predicate_coin_amount = 100;

            // Predicate has no funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);
            wallet
                .transfer(
                    predicate.address(),
                    predicate_coin_amount,
                    *base_asset_id,
                    TxPolicies::default(),
                )
                .await
                .unwrap();

            // Predicate has funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance as usize, predicate_coin_amount as usize);

            // Inputs for predicate
            let predicate_input = predicate
                .get_asset_inputs_for_amount(*base_asset_id, 1, None)
                .await
                .unwrap();

            // Outputs for predicate
            let predicate_output =
                wallet.get_asset_outputs_for_amount(&wallet.address(), *base_asset_id, 1);

            // Append the predicate to the transaction
            builder.inputs.push(predicate_input.get(0).unwrap().clone());
            builder
                .outputs
                .push(predicate_output.get(0).unwrap().clone());

            wallet.add_witnesses(&mut builder).unwrap();
            wallet.adjust_for_fee(&mut builder, 0).await.unwrap();

            // Submit the transaction
            let tx = builder.build(&provider).await.unwrap();
            provider.send_transaction_and_await_commit(tx).await.unwrap();

            // The predicate has spent it's funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);
        }

        mod message {
            use fuels::types::{coin_type::CoinType, transaction_builders::TransactionBuilder};

            use super::*;

            #[tokio::test]
            async fn can_get_input_message_sender() {
                let (contract_instance, _, wallet, _) = get_contracts(false).await;

                let message = &wallet.get_messages().await.unwrap()[0];
                let response = contract_instance
                    .methods()
                    .get_input_message_sender(0)
                    .with_inputs(vec![SdkInput::ResourceSigned {
                        resource: CoinType::Message(message.clone()),
                    }])
                    .call()
                    .await
                    .unwrap();
                assert_eq!(
                    response.value.unwrap().as_slice(),
                    message.sender.hash().as_slice()
                );

                // Assert none returned when transaction type is not a message
                let none_result = contract_instance
                    .methods()
                    .get_input_message_sender(0)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
            }

            #[tokio::test]
            async fn can_get_input_message_recipient() {
                let (contract_instance, _, wallet, _) = get_contracts(false).await;

                let message = &wallet.get_messages().await.unwrap()[0];
                let recipient = message.recipient.hash;

                let response = contract_instance
                    .methods()
                    .get_input_message_recipient(0)
                    .with_inputs(vec![SdkInput::ResourceSigned {
                        resource: CoinType::Message(message.clone()),
                    }])
                    .call()
                    .await
                    .unwrap();
                assert_eq!(response.value.unwrap().as_slice(), recipient.as_slice());

                // Assert none returned when transaction type is not a message
                let none_result = contract_instance
                    .methods()
                    .get_input_message_recipient(0)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
            }

            #[tokio::test]
            async fn can_get_input_message_nonce() {
                let (contract_instance, _, wallet, _) = get_contracts(false).await;

                let message = &wallet.get_messages().await.unwrap()[0];
                let nonce = message.nonce;

                let response = contract_instance
                    .methods()
                    .get_input_message_nonce(0)
                    .with_inputs(vec![SdkInput::ResourceSigned {
                        resource: CoinType::Message(message.clone()),
                    }])
                    .call()
                    .await
                    .unwrap();
                assert_eq!(response.value.unwrap().0.as_slice(), nonce.as_slice());

                // Assert none returned when transaction type is not a message
                let none_result = contract_instance
                    .methods()
                    .get_input_message_nonce(0)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
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
                assert_eq!(result.value, Some(0));

                // Assert none returned when not a valid index
                let none_result = contract_instance
                    .methods()
                    .get_input_witness_index(0)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
            }

            #[tokio::test]
            async fn can_get_input_message_data_length() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;

                let message = &wallet.get_messages().await.unwrap()[0];

                let response = contract_instance
                    .methods()
                    .get_input_message_data_length(0)
                    .with_inputs(vec![SdkInput::ResourceSigned {
                        resource: CoinType::Message(message.clone()),
                    }])
                    .call()
                    .await
                    .unwrap();
                assert_eq!(response.value.unwrap(), 3);

                // Assert none returned when transaction type is not a message
                let none_result = contract_instance
                    .methods()
                    .get_input_message_data_length(0)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_length() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let (predicate_bytecode, message, _) =
                    generate_predicate_inputs(100, &wallet).await;

                let response = contract_instance
                    .methods()
                    .get_input_predicate_length(0)
                    .with_inputs(vec![message])
                    .call()
                    .await
                    .unwrap();
                assert_eq!(response.value.unwrap(), predicate_bytecode.len() as u64);

                // Assert none returned when index is invalid
                let none_result = contract_instance
                    .methods()
                    .get_input_predicate_length(0) // always a contract input
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate_data_length() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let (_, message, _) = generate_predicate_inputs(100, &wallet).await;
                let provider = wallet.provider();

                let mut builder = contract_instance
                    .methods()
                    .get_input_predicate_data_length(0)
                    .with_inputs(vec![message])
                    .transaction_builder()
                    .await
                    .unwrap();

                wallet.adjust_for_fee(&mut builder, 1000).await.unwrap();
                builder.add_signer(wallet.signer().clone()).unwrap();
                let tx = builder.build(provider).await.unwrap();

                let tx_status = provider.send_transaction_and_await_commit(tx).await.unwrap();
                let receipts = tx_status
                    .take_receipts_checked(None)
                    .unwrap();

                assert_eq!(
                    receipts[1].data().unwrap()[8..16],
                    *0u64.to_le_bytes().as_slice()
                );

                // Assert none returned when transaction type is not a message
                let none_result = contract_instance
                    .methods()
                    .get_input_predicate_data_length(0)
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, None);
            }

            #[tokio::test]
            async fn can_get_input_message_data() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let message = &wallet.get_messages().await.unwrap()[0];

                // Assert none returned when transaction type is not a message
                let response = contract_instance
                    .methods()
                    .get_input_message_data(0, 0, Bytes(MESSAGE_DATA.into()))
                    .with_inputs(vec![SdkInput::ResourceSigned {
                        resource: CoinType::Message(message.clone()),
                    }])
                    .call()
                    .await
                    .unwrap();
                assert!(response.value);

                // Assert none returned when transaction type is not a message
                let none_result = contract_instance
                    .methods()
                    .get_input_message_data(3, 0, Bytes(MESSAGE_DATA.into()))
                    .call()
                    .await
                    .unwrap();
                assert_eq!(none_result.value, false);
            }

            #[tokio::test]
            async fn can_get_input_message_data_with_offset() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let message = &wallet.get_messages().await.unwrap()[0];

                let response = contract_instance
                    .methods()
                    .get_input_message_data(0, 1, Bytes(MESSAGE_DATA[1..].into()))
                    .with_inputs(vec![SdkInput::ResourceSigned {
                        resource: CoinType::Message(message.clone()),
                    }])
                    .call()
                    .await
                    .unwrap();
                assert!(response.value);
            }

            #[tokio::test]
            async fn can_get_input_message_predicate() {
                let (contract_instance, _, wallet, _) = get_contracts(true).await;
                let (predicate_bytecode, message, _) =
                    generate_predicate_inputs(100, &wallet).await;

                let response = contract_instance
                    .methods()
                    .get_input_predicate(0, predicate_bytecode.clone())
                    .with_inputs(vec![message])
                    .call()
                    .await
                    .unwrap();
                assert!(response.value);

                // Assert none returned when index is invalid
                let none_result = contract_instance
                    .methods()
                    .get_input_predicate(0, predicate_bytecode) // 0 is always a contract input
                    .call()
                    .await
                    .unwrap();

                assert_eq!(none_result.value, false);
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
            assert_eq!(result.value, Some(Output::Contract));

            // Assert invalid index returns None
            let result = contract_instance
                .methods()
                .get_output_type(2)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, None);
        }

        #[tokio::test]
        async fn can_get_tx_output_type_for_contract_deployment() {
            // Setup Wallet
            let mut node_config = NodeConfig::default();
            node_config.starting_gas_price = 0;
            let wallet = launch_custom_provider_and_get_wallets(
                WalletsConfig::new(
                    Some(1),             /* Single wallet */
                    Some(1),             /* Single coin (UTXO) */
                    Some(1_000_000_000), /* Amount per coin */
                ),
                Some(node_config),
                None,
            )
            .await
            .unwrap()
            .pop()
            .unwrap();
            let provider = wallet.try_provider().unwrap();
            let consensus_params = provider.consensus_parameters().await.unwrap();
            let base_asset_id = consensus_params.base_asset_id();

            // Get the predicate
            let predicate: Predicate =
                Predicate::load_from(TX_CONTRACT_CREATION_PREDICATE_BYTECODE_PATH)
                    .unwrap()
                    .with_provider(provider.clone());
            let predicate_coin_amount = 100;

            // Predicate has no funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);

            // Transfer funds to predicate
            wallet
                .transfer(
                    predicate.address(),
                    predicate_coin_amount,
                    *base_asset_id,
                    TxPolicies::default(),
                )
                .await
                .unwrap();

            // Predicate has funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, predicate_coin_amount);

            // Get contract ready for deployment
            let binary = fs::read(TX_CONTRACT_BYTECODE_PATH).unwrap();
            let salt = Salt::new([2u8; 32]);
            let storage_slots = Vec::<StorageSlot>::new();
            let contract = Contract::regular(binary.clone(), salt, storage_slots.clone());

            // Start building the transaction
            let tb: CreateTransactionBuilder =
                CreateTransactionBuilder::prepare_contract_deployment(
                    binary,
                    contract.contract_id(),
                    contract.state_root(),
                    salt,
                    storage_slots,
                    TxPolicies::default(),
                );

            // Inputs
            let inputs = predicate
                .get_asset_inputs_for_amount(*base_asset_id, predicate_coin_amount.into(), None)
                .await
                .unwrap();

            // Outputs
            let mut outputs = wallet.get_asset_outputs_for_amount(
                &wallet.address(),
                *base_asset_id,
                predicate_coin_amount,
            );
            outputs.push(SdkOutput::contract_created(
                contract.contract_id(),
                contract.state_root(),
            ));

            let mut tb = tb.with_inputs(inputs).with_outputs(outputs);

            wallet.add_witnesses(&mut tb).unwrap();
            wallet.adjust_for_fee(&mut tb, 1).await.unwrap();

            // Build transaction
            let tx = tb.build(provider).await.unwrap();

            // Send trandaction
            provider
                .send_transaction_and_await_commit(tx)
                .await
                .unwrap()
                .check(None)
                .unwrap();

            // Verify contract was deployed
            let instance = TxContractTest::new(contract.contract_id(), wallet.clone());
            assert!(instance.methods().get_output_type(0).call().await.is_ok());

            // Verify predicate funds transferred
            let predicate_balance = predicate
                .get_asset_balance(&AssetId::default())
                .await
                .unwrap();
            assert_eq!(predicate_balance, 0);
        }

        #[tokio::test]
        async fn can_get_tx_output_details() {
            let (wallet, _, predicate, asset_id, _) =
                setup_output_predicate(0, SwayOutput::Coin).await;

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
            let expected_fee = 1;
            assert_eq!(balance - transfer_amount - expected_fee, new_balance);
        }

        #[tokio::test]
        async fn can_get_amount_for_output_contract() {
            let (contract_instance, _, _, _) = get_contracts(true).await;
            let result = contract_instance
                .methods()
                .get_tx_output_amount(0)
                .call()
                .await
                .unwrap();
            assert_eq!(result.value, None);
        }

        #[tokio::test]
        async fn can_get_output_count_in_tx_upload() {
            // Prepare wallet and provider
            let signer = PrivateKeySigner::random(&mut rand::thread_rng());
            let num_coins = 100;
            let coins = setup_single_asset_coins(
                signer.address(),
                AssetId::zeroed(),
                num_coins,
                DEFAULT_COIN_AMOUNT,
            );
            let provider = setup_test_provider(coins, vec![], None, None)
                .await
                .unwrap();
            let wallet = Wallet::new(signer, provider.clone());
            let consensus_params = provider.consensus_parameters().await.unwrap();
            let base_asset_id = consensus_params.base_asset_id();

            // Prepare bytecode and subsections
            let bytecode = fs::read(TX_CONTRACT_BYTECODE_PATH).unwrap();
            let subsection_size = 65536;
            let subsections = UploadSubsection::split_bytecode(&bytecode, subsection_size).unwrap();

            // Upload each sub section in a separate transaction and include the predicate with the transaction.
            for subsection in subsections.clone() {
                let mut builder = UploadTransactionBuilder::prepare_subsection_upload(
                    subsection,
                    TxPolicies::default(),
                );

                // Prepare the predicate
                let predicate_data = TestTxOutputCountPredicateEncoder::default()
                    .encode_data(1) // There is only 1 output - which is a change output
                    .unwrap();
                let predicate: Predicate =
                    Predicate::load_from(TX_OUTPUT_COUNT_PREDICATE_BYTECODE_PATH)
                        .unwrap()
                        .with_provider(provider.clone())
                        .with_data(predicate_data);
                let predicate_coin_amount = 100;

                // Predicate has no funds
                let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
                assert_eq!(predicate_balance, 0);
                wallet
                    .transfer(
                        predicate.address(),
                        predicate_coin_amount,
                        *base_asset_id,
                        TxPolicies::default(),
                    )
                    .await
                    .unwrap();

                // Predicate has funds
                let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
                assert_eq!(
                    predicate_balance as usize,
                    predicate_coin_amount as usize * subsections.len()
                );

                // Inputs for predicate
                let predicate_input = predicate
                    .get_asset_inputs_for_amount(*base_asset_id, 1, None)
                    .await
                    .unwrap();

                // Append the predicate to the transaction
                builder.inputs.push(predicate_input.get(0).unwrap().clone());
                builder.outputs.push(SdkOutput::change(
                    wallet.address().into(),
                    0,
                    *base_asset_id,
                ));

                // Submit the transaction
                let tx = builder.build(&provider).await.unwrap();
                provider.send_transaction_and_await_commit(tx).await.unwrap();

                // The predicate has spent it's funds
                let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
                assert_eq!(predicate_balance, 0);
            }
        }

        #[tokio::test]
        async fn can_get_output_count_in_tx_blob() {
            // Prepare wallet and provider
            let signer = PrivateKeySigner::random(&mut rand::thread_rng());

            let num_coins = 100;
            let coins = setup_single_asset_coins(
                signer.address(),
                AssetId::zeroed(),
                num_coins,
                DEFAULT_COIN_AMOUNT,
            );
            let provider = setup_test_provider(coins, vec![], None, None)
                .await
                .unwrap();
            let wallet = Wallet::new(signer, provider.clone());
            let consensus_params = provider.consensus_parameters().await.unwrap();
            let base_asset_id = consensus_params.base_asset_id();

            // Prepare blobs
            let max_words_per_blob = 10_000;
            let blobs =
                Contract::load_from(TX_CONTRACT_BYTECODE_PATH, LoadConfiguration::default())
                    .unwrap()
                    .convert_to_loader(max_words_per_blob)
                    .unwrap()
                    .blobs()
                    .to_vec();

            let blob = blobs[0].clone();

            // Prepare the predicate
            let predicate_data = TestTxOutputCountPredicateEncoder::default()
                .encode_data(1) // There is only 1 output - which is a change output
                .unwrap();
            let predicate: Predicate =
                Predicate::load_from(TX_OUTPUT_COUNT_PREDICATE_BYTECODE_PATH)
                    .unwrap()
                    .with_provider(provider.clone())
                    .with_data(predicate_data);
            let predicate_coin_amount = 100;

            // Predicate has no funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);
            wallet
                .transfer(
                    predicate.address(),
                    predicate_coin_amount,
                    *base_asset_id,
                    TxPolicies::default(),
                )
                .await
                .unwrap();

            // Predicate has funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance as usize, predicate_coin_amount as usize);

            // Inputs for predicate
            let predicate_input = predicate
                .get_asset_inputs_for_amount(*base_asset_id, 1, None)
                .await
                .unwrap();

            let mut builder = BlobTransactionBuilder::default().with_blob(blob);

            // Append the predicate to the transaction
            builder.inputs.push(predicate_input.get(0).unwrap().clone());
            builder.outputs.push(SdkOutput::change(
                wallet.address().into(),
                0,
                *base_asset_id,
            ));

            // Submit the transaction
            let tx = builder.build(&provider).await.unwrap();
            provider.send_transaction_and_await_commit(tx).await.unwrap();

            // The predicate has spent it's funds
            let predicate_balance = predicate.get_asset_balance(base_asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);
        }

        #[tokio::test]
        async fn can_get_tx_output_change_details() {
            // Prepare predicate
            let (wallet, wallet_2, predicate, asset_id, _) =
                setup_output_predicate(2, SwayOutput::Change).await;
            let provider = wallet.provider();

            let predicate_balance_before = predicate.get_asset_balance(&asset_id).await.unwrap();
            let wallet_2_balance_before = wallet_2.get_asset_balance(&asset_id).await.unwrap();

            // Deploy contract
            let contract_id = Contract::load_from(
                TX_OUTPUT_CONTRACT_BYTECODE_PATH,
                LoadConfiguration::default(),
            )
            .unwrap()
            .deploy(&wallet, TxPolicies::default())
            .await
            .unwrap()
            .contract_id;

            let instance = TxOutputContract::new(contract_id.clone(), wallet.clone());
            // Send tokens to the contract
            let _ = wallet
                .force_transfer_to_contract(&contract_id, 10, asset_id, TxPolicies::default())
                .await
                .unwrap();

            let wallet_1_balance_before = wallet.get_asset_balance(&asset_id).await.unwrap();

            // Build transaction
            let call_handler =
                instance
                    .methods()
                    .send_assets_change(wallet.clone().address(), asset_id, 10);
            let mut tb = call_handler.transaction_builder().await.unwrap();

            // Inputs for predicate
            let transfer_amount = 50u64;
            let predicate_inputs = predicate
                .get_asset_inputs_for_amount(asset_id, transfer_amount.into(), None)
                .await
                .unwrap();

            // Outputs for predicate
            let predicate_outputs =
                wallet.get_asset_outputs_for_amount(wallet_2.address(), asset_id, transfer_amount);

            // Append the inputs and outputs to the transaction
            tb.inputs.extend(predicate_inputs);
            tb.outputs.extend(predicate_outputs);

            let tx = tb.build(provider.clone()).await.unwrap();
            let tx_status = provider
                .send_transaction_and_await_commit(tx)
                .await
                .unwrap();

            // Assert the predicate balance is empty
            let predicate_balance = predicate.get_asset_balance(&asset_id).await.unwrap();
            assert_eq!(predicate_balance, 0);

            // Assert the wallet 1 has received the change
            let wallet_1_balance = wallet.get_asset_balance(&asset_id).await.unwrap();
            let change_amount = predicate_balance_before - transfer_amount - tx_status.total_fee();
            assert_eq!(wallet_1_balance, wallet_1_balance_before + change_amount);

            // Assert the wallet 2 has received the transfer amount
            let wallet_2_balance = wallet_2.get_asset_balance(&asset_id).await.unwrap();
            assert_eq!(wallet_2_balance, wallet_2_balance_before + transfer_amount);
        }

        #[tokio::test]
        async fn can_get_tx_output_variable_details() {
            // Prepare wallet
            let (wallet, _, _, asset_id, _) = setup_output_predicate(1, SwayOutput::Variable).await;

            // Deploy contract
            let contract_id = Contract::load_from(
                TX_OUTPUT_CONTRACT_BYTECODE_PATH,
                LoadConfiguration::default(),
            )
            .unwrap()
            .deploy(&wallet, TxPolicies::default())
            .await
            .unwrap()
            .contract_id;

            let instance = TxOutputContract::new(contract_id.clone(), wallet.clone());

            // Send tokens to the contract
            let _ = wallet
                .force_transfer_to_contract(&contract_id, 10, asset_id, TxPolicies::default())
                .await
                .unwrap();

            // Run transaction with variable output
            let (tx_to, tx_asset_id, tx_amount) = instance
                .methods()
                .send_assets_variable(wallet.clone().address(), asset_id, 2)
                .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                .call()
                .await
                .unwrap()
                .value;

            assert_eq!(tx_to, wallet.clone().address().into());
            assert_eq!(tx_asset_id, asset_id);
            assert_eq!(tx_amount, 1);
        }
    }

    mod revert {
        use super::*;

        #[tokio::test]
        #[should_panic]
        async fn fails_output_predicate_when_incorrect_asset() {
            let (wallet1, _, predicate, _, asset_id2) =
                setup_output_predicate(0, SwayOutput::Coin).await;

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
            let (_, wallet2, predicate, asset_id1, _) =
                setup_output_predicate(0, SwayOutput::Coin).await;

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
