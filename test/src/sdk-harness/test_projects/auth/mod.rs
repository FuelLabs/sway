use fuels::{
    accounts::{predicate::Predicate, signers::private_key::PrivateKeySigner},
    prelude::*,
    tx::UtxoId,
    types::{
        coin::{Coin},
        coin_type::CoinType,
        input::Input,
        message::{Message, MessageStatus},
        Bytes32, ContractId,
    },
};
use std::str::FromStr;

abigen!(
    Contract(
        name = "AuthContract",
        abi = "out/auth_testing_contract-abi.json"
    ),
    Contract(
        name = "AuthCallerContract",
        abi = "out/auth_caller_contract-abi.json"
    ),
    Predicate(
        name = "AuthPredicate",
        abi = "out/auth_predicate-abi.json"
    ),
);

#[tokio::test]
async fn is_external_from_sdk() {
    let (auth_instance, _, _, _, _) = get_contracts().await;
    let result = auth_instance
        .methods()
        .is_caller_external()
        .call()
        .await
        .unwrap();

    assert!(result.value);
}

#[tokio::test]
async fn msg_sender_from_sdk() {
    let (auth_instance, _, _, _, wallet) = get_contracts().await;
    let result = auth_instance
        .methods()
        .returns_msg_sender_address(wallet.address())
        .call()
        .await
        .unwrap();

    assert!(result.value);
}

#[tokio::test]
async fn msg_sender_from_contract() {
    let (auth_instance, auth_id, caller_instance, caller_id, _) = get_contracts().await;

    let result = caller_instance
        .methods()
        .call_auth_contract(auth_id, caller_id)
        .with_contracts(&[&auth_instance])
        .call()
        .await
        .unwrap();

    assert!(result.value);
}

#[tokio::test]
async fn input_message_msg_sender_from_contract() {
    // Wallet
    let wallet_signer = PrivateKeySigner::random(&mut rand::thread_rng());
    let deployment_signer = PrivateKeySigner::random(&mut rand::thread_rng());

    // Setup coins and messages
    let coins = setup_single_asset_coins(wallet_signer.address(), AssetId::BASE, 100, 1000);
    let coins_2 = setup_single_asset_coins(deployment_signer.address(), AssetId::BASE, 100, 1000);
    let total_coins = [coins, coins_2].concat();

    let msg = setup_single_message(
        Address::default(),
        wallet_signer.address(),
        DEFAULT_COIN_AMOUNT,
        10.into(),
        vec![],
    );

    let provider = setup_test_provider(total_coins.clone(), vec![msg.clone()], None, None)
        .await
        .unwrap();
    let wallet = Wallet::new(wallet_signer, provider.clone());
    let deployer_wallet = Wallet::new(deployment_signer, provider.clone());

    // Setup contract
    let id = Contract::load_from(
        "out/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&deployer_wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let instance = AuthContract::new(id.clone(), wallet.clone());

    // Start building transactions
    let call_handler = instance.methods().returns_msg_sender_address(msg.recipient);
    let mut tb = call_handler
        .transaction_builder()
        .await
        .unwrap()
        .enable_burn(true);

    // Inputs
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Message(
            wallet
                .get_messages()
                .await
                .unwrap()
                .first()
                .unwrap()
                .clone(),
        ),
    });

    // Build transaction
    tb.add_signer(wallet.signer().clone()).unwrap();
    let tx = tb.build(provider.clone()).await.unwrap();

    // Send and verify
    let tx_status = provider.send_transaction_and_await_commit(tx).await.unwrap();
    let response = call_handler.get_response(tx_status).unwrap();
    assert!(response.value);
}

#[tokio::test]
async fn caller_addresses_from_messages() {
    let signer_1 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_2 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_3 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_4 = PrivateKeySigner::random(&mut rand::thread_rng());

    // Setup message
    let message_amount = 10;
    let message1 = Message {
        sender: signer_1.address(),
        recipient: signer_1.address(),
        nonce: 0.into(),
        amount: message_amount,
        data: vec![],
        da_height: 0,
        status: MessageStatus::Unspent,
    };
    let message2 = Message {
        sender: signer_2.address(),
        recipient: signer_2.address(),
        nonce: 1.into(),
        amount: message_amount,
        data: vec![],
        da_height: 0,
        status: MessageStatus::Unspent,
    };
    let message3 = Message {
        sender: signer_3.address(),
        recipient: signer_3.address(),
        nonce: 2.into(),
        amount: message_amount,
        data: vec![],
        da_height: 0,
        status: MessageStatus::Unspent,
    };
    let mut message_vec: Vec<Message> = Vec::new();
    message_vec.push(message1);
    message_vec.push(message2);
    message_vec.push(message3);

    // Setup Coin
    let coin_amount = 10;
    let coin = Coin {
        owner: signer_4.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let provider = setup_test_provider(vec![coin], message_vec, Some(node_config), None)
        .await
        .unwrap();
    let wallet1 = Wallet::new(signer_1, provider.clone());
    let wallet2 = Wallet::new(signer_2, provider.clone());
    let wallet3 = Wallet::new(signer_3, provider.clone());
    let wallet4 = Wallet::new(signer_4, provider.clone());

    let id_1 = Contract::load_from(
        "out/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet4, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let auth_instance = AuthContract::new(id_1.clone(), wallet4.clone());

    let result = auth_instance
        .methods()
        .returns_caller_addresses()
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, vec![Address::from(*wallet4.address())]);

    // Start building transactions
    let call_handler = auth_instance.methods().returns_caller_addresses();
    let mut tb = call_handler.transaction_builder().await.unwrap();

    // Inputs
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Message(setup_single_message(
            wallet1.address(),
            wallet1.address(),
            message_amount,
            0.into(),
            vec![],
        )),
    });
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Message(setup_single_message(
            wallet2.address(),
            wallet2.address(),
            message_amount,
            1.into(),
            vec![],
        )),
    });
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Message(setup_single_message(
            wallet3.address(),
            wallet3.address(),
            message_amount,
            2.into(),
            vec![],
        )),
    });

    // Build transaction
    tb.add_signer(wallet1.signer().clone()).unwrap();
    tb.add_signer(wallet2.signer().clone()).unwrap();
    tb.add_signer(wallet3.signer().clone()).unwrap();

    let provider = wallet1.provider();
    let tx = tb.enable_burn(true).build(provider.clone()).await.unwrap();

    // Send and verify
    let tx_status = provider.send_transaction_and_await_commit(tx).await.unwrap();
    let result = call_handler.get_response(tx_status).unwrap();

    assert!(result
        .value
        .contains(&Address::from(wallet1.address())));
    assert!(result
        .value
        .contains(&Address::from(wallet2.address())));
    assert!(result
        .value
        .contains(&Address::from(wallet3.address())));
}

#[tokio::test]
async fn caller_addresses_from_coins() {
    let signer_1 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_2 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_3 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_4 = PrivateKeySigner::random(&mut rand::thread_rng());

    // Setup Coin
    let coin_amount = 10;
    let coin1 = Coin {
        owner: signer_1.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };
    let coin2 = Coin {
        owner: signer_2.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 1),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };
    let coin3 = Coin {
        owner: signer_3.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 2),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };
    let coin4 = Coin {
        owner: signer_4.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 3),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };

    let mut coin_vec: Vec<Coin> = Vec::new();
    coin_vec.push(coin1);
    coin_vec.push(coin2);
    coin_vec.push(coin3);
    coin_vec.push(coin4);

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let provider = setup_test_provider(coin_vec, vec![], Some(node_config), None)
        .await
        .unwrap();
    let wallet1 = Wallet::new(signer_1, provider.clone());
    let wallet2 = Wallet::new(signer_2, provider.clone());
    let wallet3 = Wallet::new(signer_3, provider.clone());
    let wallet4 = Wallet::new(signer_4, provider.clone());

    let id_1 = Contract::load_from(
        "out/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet4, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let auth_instance = AuthContract::new(id_1.clone(), wallet4.clone());

    let result = auth_instance
        .methods()
        .returns_caller_addresses()
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, vec![Address::from(*wallet4.address())]);

    // Start building transactions
    let call_handler = auth_instance.methods().returns_caller_addresses();
    let mut tb = call_handler.transaction_builder().await.unwrap();

    // Inputs
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Coin(Coin {
            owner: wallet1.address(),
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            amount: coin_amount,
            asset_id: AssetId::default(),
                }),
    });
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Coin(Coin {
            owner: wallet2.address(),
            utxo_id: UtxoId::new(Bytes32::zeroed(), 1),
            amount: coin_amount,
            asset_id: AssetId::default(),
                }),
    });
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Coin(Coin {
            owner: wallet3.address(),
            utxo_id: UtxoId::new(Bytes32::zeroed(), 2),
            amount: coin_amount,
            asset_id: AssetId::default(),
                }),
    });

    // Build transaction
    tb.add_signer(wallet1.signer().clone()).unwrap();
    tb.add_signer(wallet2.signer().clone()).unwrap();
    tb.add_signer(wallet3.signer().clone()).unwrap();

    let provider = wallet1.provider();
    let tx = tb.enable_burn(true).build(provider.clone()).await.unwrap();

    // Send and verify
    let tx_status = provider.send_transaction_and_await_commit(tx).await.unwrap();
    let result = call_handler.get_response(tx_status).unwrap();

    assert!(result
        .value
        .contains(&Address::from(wallet1.address())));
    assert!(result
        .value
        .contains(&Address::from(wallet2.address())));
    assert!(result
        .value
        .contains(&Address::from(wallet3.address())));
}

#[tokio::test]
async fn caller_addresses_from_coins_and_messages() {
    let signer_1 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_2 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_3 = PrivateKeySigner::random(&mut rand::thread_rng());
    let signer_4 = PrivateKeySigner::random(&mut rand::thread_rng());

    let message_amount = 10;
    let message1 = Message {
        sender: signer_1.address(),
        recipient: signer_1.address(),
        nonce: 0.into(),
        amount: message_amount,
        data: vec![],
        da_height: 0,
        status: MessageStatus::Unspent,
    };

    // Setup Coin
    let coin_amount = 10;
    let coin2 = Coin {
        owner: signer_2.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 1),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };
    let coin3 = Coin {
        owner: signer_3.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 2),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };
    let coin4 = Coin {
        owner: signer_4.address(),
        utxo_id: UtxoId::new(Bytes32::zeroed(), 3),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };

    let mut coin_vec: Vec<Coin> = Vec::new();
    coin_vec.push(coin2);
    coin_vec.push(coin3);
    coin_vec.push(coin4);

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let provider = setup_test_provider(coin_vec, vec![message1], Some(node_config), None)
        .await
        .unwrap();

    let wallet1 = Wallet::new(signer_1, provider.clone());
    let wallet2 = Wallet::new(signer_2, provider.clone());
    let wallet3 = Wallet::new(signer_3, provider.clone());
    let wallet4 = Wallet::new(signer_4, provider.clone());

    let id_1 = Contract::load_from(
        "out/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet4, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let auth_instance = AuthContract::new(id_1.clone(), wallet4.clone());

    let result = auth_instance
        .methods()
        .returns_caller_addresses()
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, vec![Address::from(*wallet4.address())]);

    // Start building transactions
    let call_handler = auth_instance.methods().returns_caller_addresses();
    let mut tb = call_handler.transaction_builder().await.unwrap();

    // Inputs
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Message(setup_single_message(
            wallet1.address(),
            wallet1.address(),
            message_amount,
            0.into(),
            vec![],
        )),
    });
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Coin(Coin {
            owner: wallet2.address(),
            utxo_id: UtxoId::new(Bytes32::zeroed(), 1),
            amount: coin_amount,
            asset_id: AssetId::default(),
                }),
    });
    tb.inputs_mut().push(Input::ResourceSigned {
        resource: CoinType::Coin(Coin {
            owner: wallet3.address(),
            utxo_id: UtxoId::new(Bytes32::zeroed(), 2),
            amount: coin_amount,
            asset_id: AssetId::default(),
                }),
    });

    // Build transaction
    tb.add_signer(wallet1.signer().clone()).unwrap();
    tb.add_signer(wallet2.signer().clone()).unwrap();
    tb.add_signer(wallet3.signer().clone()).unwrap();

    let provider = wallet1.provider();
    let tx = tb.enable_burn(true).build(provider.clone()).await.unwrap();

    // Send and verify
    let tx_status = provider.send_transaction_and_await_commit(tx).await.unwrap();
    let result = call_handler.get_response(tx_status).unwrap();

    assert!(result
        .value
        .contains(&Address::from(wallet1.address())));
    assert!(result
        .value
        .contains(&Address::from(wallet2.address())));
    assert!(result
        .value
        .contains(&Address::from(wallet3.address())));
}

async fn get_contracts() -> (
    AuthContract<Wallet>,
    ContractId,
    AuthCallerContract<Wallet>,
    ContractId,
    Wallet,
) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let id_1 = Contract::load_from(
        "out/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let id_2 = Contract::load_from(
        "out/auth_caller_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let instance_1 = AuthContract::new(id_1.clone(), wallet.clone());
    let instance_2 = AuthCallerContract::new(id_2.clone(), wallet.clone());

    (instance_1, id_1.into(), instance_2, id_2.into(), wallet)
}

#[tokio::test]
async fn can_get_predicate_address() {
    // Setup Wallets
    let asset_id = AssetId::default();
    let wallets_config = WalletsConfig::new_multiple_assets(
        2,
        vec![AssetConfig {
            id: asset_id,
            num_coins: 1,
            coin_amount: 1_000,
        }],
    );
    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let wallets = &launch_custom_provider_and_get_wallets(wallets_config, Some(node_config), None)
        .await
        .unwrap();
    let first_wallet = &wallets[0];
    let second_wallet = &wallets[1];

    // Setup predicate.
    let hex_predicate_address: &str =
        "0xf9acc710533729d33e311df5dcbddca07898135691656fc6a95c77fdb36b0940";
    let predicate_address =
        Address::from_str(hex_predicate_address).expect("failed to create Address from string");
    let predicate_data = AuthPredicateEncoder::default()
        .encode_data(predicate_address)
        .unwrap();
    let predicate: Predicate =
        Predicate::load_from("out/auth_predicate.bin")
            .unwrap()
            .with_provider(first_wallet.try_provider().unwrap().clone())
            .with_data(predicate_data);

    // If this test fails, it can be that the predicate address got changed.
    // Uncomment the next line, get the predicate address, and update it above.
    // dbg!(&predicate);

    // Next, we lock some assets in this predicate using the first wallet:
    // First wallet transfers amount to predicate.
    first_wallet
        .transfer(predicate.address(), 500, asset_id, TxPolicies::default())
        .await
        .unwrap();

    // Check predicate balance.
    let balance = predicate
        .get_asset_balance(&AssetId::default())
        .await
        .unwrap();
    assert_eq!(balance, 500);

    // Then we can transfer assets owned by the predicate via the Account trait:
    let amount_to_unlock = 500;

    // Will transfer if the correct predicate address is passed as an argument to the predicate
    predicate
        .transfer(
            second_wallet.address(),
            amount_to_unlock,
            asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // Predicate balance is zero.
    let balance = predicate
        .get_asset_balance(&AssetId::default())
        .await
        .unwrap();
    assert_eq!(balance, 0);

    // Second wallet balance is updated.
    let balance = second_wallet
        .get_asset_balance(&AssetId::default())
        .await
        .unwrap();
    assert_eq!(balance, 1500);
}

#[tokio::test]
#[should_panic]
async fn when_incorrect_predicate_address_passed() {
    // Setup Wallets
    let asset_id = AssetId::default();
    let wallets_config = WalletsConfig::new_multiple_assets(
        2,
        vec![AssetConfig {
            id: asset_id,
            num_coins: 1,
            coin_amount: 1_000,
        }],
    );
    let wallets = &launch_custom_provider_and_get_wallets(wallets_config, None, None)
        .await
        .unwrap();
    let first_wallet = &wallets[0];
    let second_wallet = &wallets[1];

    // Setup predicate with incorrect address.
    let hex_predicate_address: &str =
        "0x61c2fbc40e1fe1602f8734928a6f1bf76d5d70f9c0407e6dc74e4bfdfb7ac392";
    let predicate_address =
        Address::from_str(hex_predicate_address).expect("failed to create Address from string");
    let predicate_data = AuthPredicateEncoder::default()
        .encode_data(predicate_address)
        .unwrap();
    let predicate: Predicate =
        Predicate::load_from("out/auth_predicate.bin")
            .unwrap()
            .with_provider(first_wallet.try_provider().unwrap().clone())
            .with_data(predicate_data);

    // Next, we lock some assets in this predicate using the first wallet:
    // First wallet transfers amount to predicate.
    first_wallet
        .transfer(predicate.address(), 500, asset_id, TxPolicies::default())
        .await
        .unwrap();

    // Check predicate balance.
    let balance = predicate
        .get_asset_balance(&AssetId::default())
        .await
        .unwrap();
    assert_eq!(balance, 500);

    // Then we can transfer assets owned by the predicate via the Account trait:
    let amount_to_unlock = 500;

    // Will should fail to transfer
    predicate
        .transfer(
            second_wallet.address(),
            amount_to_unlock,
            asset_id,
            TxPolicies::default(),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn can_get_predicate_address_in_message() {
    // Setup predicate address.
    let hex_predicate_address: &str =
        "0xf9acc710533729d33e311df5dcbddca07898135691656fc6a95c77fdb36b0940";
    let predicate_address =
        Address::from_str(hex_predicate_address).expect("failed to create Address from string");

    // Setup message
    let message_amount = 1;
    let message = Message {
        sender: Address::default(),
        recipient: predicate_address,
        nonce: 0.into(),
        amount: message_amount,
        data: vec![],
        da_height: 0,
        status: MessageStatus::Unspent,
    };
    let mut message_vec: Vec<Message> = Vec::new();
    message_vec.push(message);

    // Setup Coin
    let coin_amount = 0;
    let coin = Coin {
        owner: predicate_address,
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
        amount: coin_amount,
        asset_id: AssetId::default(),
    };
    let mut coin_vec: Vec<Coin> = Vec::new();
    coin_vec.push(coin);

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let provider = setup_test_provider(coin_vec, message_vec, Some(node_config), None)
        .await
        .unwrap();
    let wallet = Wallet::random(&mut rand::thread_rng(), provider);

    // Setup predicate.
    let predicate_data = AuthPredicateEncoder::default()
        .encode_data(predicate_address)
        .unwrap();
    let predicate: Predicate =
        Predicate::load_from("out/auth_predicate.bin")
            .unwrap()
            .with_provider(wallet.try_provider().unwrap().clone())
            .with_data(predicate_data);

    // If this test fails, it can be that the predicate address got changed.
    // Uncomment the next line, get the predicate address, and update it above.
    // dbg!(&predicate);

    // Check predicate balance.
    let balance = predicate
        .get_asset_balance(&AssetId::default())
        .await
        .unwrap();
    assert_eq!(balance, message_amount as u128);

    // Spend the message
    predicate
        .transfer(
            wallet.address(),
            message_amount,
            AssetId::default(),
            TxPolicies::default(),
        )
        .await
        .unwrap();

    // The predicate has spent the funds
    let predicate_balance = predicate
        .get_asset_balance(&AssetId::default())
        .await
        .unwrap();
    assert_eq!(predicate_balance, 0);

    // Funds were transferred
    let wallet_balance = wallet.get_asset_balance(&AssetId::default()).await.unwrap();
    assert_eq!(wallet_balance, message_amount as u128);
}
