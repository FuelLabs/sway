use fuels::{
    accounts::wallet::{Wallet, WalletUnlocked},
    prelude::*,
    types::ContractId,
};

abigen!(
    Contract(
        name = "AuthContract",
        abi = "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract-abi.json"
    ),
    Contract(
        name = "AuthCallerContract",
        abi = "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-abi.json"
    ),
    Predicate(
        name="AuthPredicate", 
        abi="test_artifacts/auth_predicate/out/debug/auth_predicate-abi.json"
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

async fn get_contracts() -> (
    AuthContract<WalletUnlocked>,
    ContractId,
    AuthCallerContract<WalletUnlocked>,
    ContractId,
    Wallet,
) {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let id_1 = Contract::load_from(
        "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let id_2 = Contract::load_from(
        "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance_1 = AuthContract::new(id_1.clone(), wallet.clone());
    let instance_2 = AuthCallerContract::new(id_2.clone(), wallet.clone());

    (
        instance_1,
        id_1.into(),
        instance_2,
        id_2.into(),
        wallet.lock(),
    )
}

#[tokio::test]
async fn can_get_predicate_id() {
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
    let wallets = &launch_custom_provider_and_get_wallets(wallets_config, None, None).await.unwrap();
    let first_wallet = &wallets[0];
    let second_wallet = &wallets[1];

    // Setup Predciate
    let hex_predicate_address: &str = "0x935a191561e0e9388c1b5dfc4626f6ecd98b2b3dd416a9a9e9ce2f0bb214f4b8";
    let predicate_address = Address::from_str(hex_predicate_address).expect("failed to create Address from string");
    let predicate_data = AuthPredicateEncoder::encode_data(Bech32Address::from(predicate_address));
    let predicate: Predicate = Predicate::load_from("test_artifacts/auth_predicate/out/debug/auth_predicate.bin").unwrap()
        .with_provider(first_wallet.try_provider().unwrap().clone())
        .with_data(predicate_data);

    // Assert predicate addresses are the same
    assert_eq!(predicate.address(), predicate_address);

    // Next, we lock some assets in this predicate using the first wallet:
    // First wallet transfers amount to predicate.
    first_wallet
        .transfer(predicate.address(), 500, asset_id, TxPolicies::default())
        .await.unwrap();

    // Check predicate balance.
    let balance = predicate.get_asset_balance(&AssetId::default()).await.unwrap();
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
        .await.unwrap();

    // Predicate balance is zero.
    let balance = predicate.get_asset_balance(&AssetId::default()).await.unwrap();
    assert_eq!(balance, 0);

    // Second wallet balance is updated.
    let balance = second_wallet.get_asset_balance(&AssetId::default()).await.unwrap();
    assert_eq!(balance, 1500);
}

#[tokio::test]
#[should_fail]
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
    let wallets = &launch_custom_provider_and_get_wallets(wallets_config, None, None).await.unwrap();
    let first_wallet = &wallets[0];
    let second_wallet = &wallets[1];

    // Setup Predciate with incorrect address
    let hex_predicate_address: &str = "0x36bf4bd40f2a3b3db595ef8fd8b21dbe9e6c0dd7b419b4413ff6b584ce7da5d7";
    let predicate_address = Address::from_str(hex_predicate_address).expect("failed to create Address from string");
    let predicate_data = AuthPredicateEncoder::encode_data(Bech32Address::from(predicate_address));
    let predicate: Predicate = Predicate::load_from("test_artifacts/auth_predicate/out/debug/auth_predicate.bin").unwrap()
        .with_provider(first_wallet.try_provider().unwrap().clone())
        .with_data(predicate_data);

    // Next, we lock some assets in this predicate using the first wallet:
    // First wallet transfers amount to predicate.
    first_wallet
        .transfer(predicate.address(), 500, asset_id, TxPolicies::default())
        .await.unwrap();

    // Check predicate balance.
    let balance = predicate.get_asset_balance(&AssetId::default()).await.unwrap();
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
        .await.unwrap();
}
