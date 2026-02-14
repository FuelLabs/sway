use fuels::{
    prelude::*,
    types::{Bits256, Bytes32, Identity},
};
use sha2::{Digest, Sha256};
use std::str::FromStr;

abigen!(Contract(
    name = "TestFuelCoinContract",
    abi = "out/asset_ops-abi.json"
));

#[tokio::test]
async fn can_mint() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet).await;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;

    let mut balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcontract_instance
        .methods()
        .mint_coins(11, Bits256(*sub_id))
        .call()
        .await
        .unwrap();

    balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 11);
}

#[tokio::test]
async fn can_mint_multiple() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet).await;
    let sub_id_1 = Bytes32::zeroed();
    let sub_id_2 = Bytes32::from([1u8; 32]);
    let asset_id_1 = get_asset_id(sub_id_1, fuelcontract_id).await;
    let asset_id_2 = get_asset_id(sub_id_2, fuelcontract_id).await;

    let mut balance_result_1 = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id_1), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result_1.value, 0);

    let mut balance_result_2 = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id_2), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result_2.value, 0);

    fuelcontract_instance
        .methods()
        .mint_coins(11, Bits256(*sub_id_1))
        .call()
        .await
        .unwrap();

    fuelcontract_instance
        .methods()
        .mint_coins(12, Bits256(*sub_id_2))
        .call()
        .await
        .unwrap();

    balance_result_1 = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id_1), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result_1.value, 11);

    balance_result_2 = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id_2), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result_2.value, 12);
}

#[tokio::test]
async fn can_burn() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet).await;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;

    let mut balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcontract_instance
        .methods()
        .mint_coins(11, Bits256(*sub_id))
        .call()
        .await
        .unwrap();
    fuelcontract_instance
        .methods()
        .burn_coins(7, Bits256(*sub_id))
        .call()
        .await
        .unwrap();

    balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 4);
}

#[tokio::test]
async fn can_force_transfer() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet.clone()).await;
    let balance_id = get_balance_contract_id(wallet).await;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;

    let mut balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcontract_instance
        .methods()
        .mint_coins(100, Bits256(*sub_id))
        .call()
        .await
        .unwrap();

    balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 100);

    // confirm initial balance on balance contract (recipient)
    balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), balance_id)
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    let coins = 42u64;

    fuelcontract_instance
        .methods()
        .force_transfer_coins(coins, Bits256(*asset_id), balance_id)
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    // confirm remaining balance on fuelcoin contract
    balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), fuelcontract_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 58);

    // confirm new balance on balance contract (recipient)
    balance_result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), balance_id)
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 42);
}

#[tokio::test]
async fn can_mint_and_send_to_contract() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet.clone()).await;
    let balance_id = get_balance_contract_id(wallet).await;
    let amount = 55u64;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;

    fuelcontract_instance
        .methods()
        .mint_and_send_to_contract(amount, balance_id, Bits256(*sub_id))
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    let result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), balance_id)
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, amount)
}

#[tokio::test]
async fn can_mint_and_send_to_address() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet.clone()).await;
    let amount = 55u64;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;
    let asset_id_array: [u8; 32] = *asset_id;

    let address = wallet.address();
    let recipient = address.clone();

    fuelcontract_instance
        .methods()
        .mint_and_send_to_address(amount, recipient, Bits256(*sub_id))
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    assert_eq!(
        wallet
            .get_spendable_resources(AssetId::from(asset_id_array), 1, None)
            .await
            .unwrap()[0]
            .amount(),
        amount
    );
}

#[tokio::test]
async fn can_perform_generic_mint_to_with_address() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet.clone()).await;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;
    let amount = 55u64;
    let asset_id_array: [u8; 32] = *asset_id;
    let address = wallet.address();

    fuelcontract_instance
        .methods()
        .generic_mint_to(amount, Identity::Address(address.into()), Bits256(*sub_id))
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    assert_eq!(
        wallet
            .get_spendable_resources(AssetId::from(asset_id_array), 1, None)
            .await
            .unwrap()[0]
            .amount(),
        amount
    );
}

#[tokio::test]
async fn can_perform_generic_mint_to_with_contract_id() {
    let num_wallets = 1;
    let coins_per_wallet = 1;
    let amount_per_coin = 1_000_000;

    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );

    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallets[0].clone()).await;
    let balance_id = get_balance_contract_id(wallets[0].clone()).await;
    let amount = 55u64;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;

    fuelcontract_instance
        .methods()
        .generic_mint_to(amount, Identity::ContractId(balance_id), Bits256(*sub_id))
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    let result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), balance_id)
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, amount)
}

#[tokio::test]
async fn can_perform_generic_transfer_to_address() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallet.clone()).await;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;
    let amount = 33u64;
    let asset_id_array: [u8; 32] = *asset_id;
    let address = wallet.address();

    fuelcontract_instance
        .methods()
        .mint_coins(amount, Bits256(*sub_id))
        .call()
        .await
        .unwrap();

    fuelcontract_instance
        .methods()
        .generic_transfer(
            amount,
            Bits256(*asset_id),
            Identity::Address(address.into()),
        )
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    assert_eq!(
        wallet
            .get_spendable_resources(AssetId::from(asset_id_array), 1, None)
            .await
            .unwrap()[0]
            .amount(),
        amount
    );
}

#[tokio::test]
async fn can_perform_generic_transfer_to_contract() {
    let num_wallets = 1;
    let coins_per_wallet = 1;
    let amount_per_coin = 1_000_000;

    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );
    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();

    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallets[0].clone()).await;
    let balance_id = get_balance_contract_id(wallets[0].clone()).await;
    let sub_id = Bytes32::zeroed();
    let asset_id = get_asset_id(sub_id, fuelcontract_id).await;
    let amount = 44u64;

    fuelcontract_instance
        .methods()
        .mint_coins(amount, Bits256(*sub_id))
        .call()
        .await
        .unwrap();

    fuelcontract_instance
        .methods()
        .generic_transfer(amount, Bits256(*asset_id), Identity::ContractId(balance_id))
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    let result = fuelcontract_instance
        .methods()
        .get_balance(Bits256(*asset_id), balance_id)
        .with_contract_ids(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, amount)
}

#[tokio::test]
async fn can_send_message_output_with_data() {
    let num_wallets = 1;
    let coins_per_wallet = 1;
    let amount_per_coin = 1_000_000;

    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );

    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallets[0].clone()).await;

    let amount = 33u64;
    let recipient_address: Address = wallets[0].address().into();

    let call_response = fuelcontract_instance
        .methods()
        .send_message(Bits256(*recipient_address), vec![100, 75, 50], amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*fuelcontract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(3, message_receipt.len().unwrap());
    assert_eq!(vec![100, 75, 50], message_receipt.data().unwrap());
}

#[tokio::test]
async fn can_send_message_output_without_data() {
    let num_wallets = 1;
    let coins_per_wallet = 1;
    let amount_per_coin = 1_000_000;

    let config = WalletsConfig::new(
        Some(num_wallets),
        Some(coins_per_wallet),
        Some(amount_per_coin),
    );

    let wallets = launch_custom_provider_and_get_wallets(config, None, None)
        .await
        .unwrap();
    let (fuelcontract_instance, fuelcontract_id) = get_fuelcoin_instance(wallets[0].clone()).await;

    let amount = 33u64;
    let recipient_hex = "0x000000000000000000000000b46a7a1a23f3897cc83a94521a96da5c23bc58db";
    let recipient_address = Address::from_str(recipient_hex).unwrap();

    let call_response = fuelcontract_instance
        .methods()
        .send_message(Bits256(*recipient_address), Vec::<u64>::new(), amount)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .tx_status
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*fuelcontract_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(0, message_receipt.len().unwrap());
    assert_eq!(Vec::<u8>::new(), message_receipt.data().unwrap());
}

async fn get_fuelcoin_instance(wallet: Wallet) -> (TestFuelCoinContract<Wallet>, ContractId) {
    let fuelcontract_id = Contract::load_from(
        "out/asset_ops.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    wallet
        .force_transfer_to_contract(fuelcontract_id, 1000, AssetId::BASE, TxPolicies::default())
        .await
        .unwrap();
    let fuelcontract_instance = TestFuelCoinContract::new(fuelcontract_id.clone(), wallet);

    (fuelcontract_instance, fuelcontract_id.into())
}

async fn get_balance_contract_id(wallet: Wallet) -> ContractId {
    let balance_id = Contract::load_from(
        "out/balance_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    balance_id.into()
}

async fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> Bytes32 {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    Bytes32::from(<[u8; 32]>::from(hasher.finalize()))
}
