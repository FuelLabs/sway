use fuels::{prelude::*, tx::AssetId};
use std::str::FromStr;

abigen!(
    TestFuelCoinContract,
    "test_projects/token_ops/out/debug/token_ops-abi.json"
);

#[tokio::test]
async fn can_mint() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet).await;

    let target = fuelcoin_id.clone();
    let asset_id = target.clone();

    let mut balance_result = fuelcoin_instance
        .methods()
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance
        .methods()
        .mint_coins(11)
        .call()
        .await
        .unwrap();

    balance_result = fuelcoin_instance
        .methods()
        .get_balance(target, asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 11);
}

#[tokio::test]
async fn can_burn() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet).await;

    let target = fuelcoin_id.clone();
    let asset_id = target.clone();

    let mut balance_result = fuelcoin_instance
        .methods()
        .get_balance(target.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance
        .methods()
        .mint_coins(11)
        .call()
        .await
        .unwrap();
    fuelcoin_instance
        .methods()
        .burn_coins(7)
        .call()
        .await
        .unwrap();

    balance_result = fuelcoin_instance
        .methods()
        .get_balance(target, asset_id)
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 4);
}

#[tokio::test]
async fn can_force_transfer() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet.clone()).await;
    let balance_id = get_balance_contract_id(wallet).await;

    let asset_id = fuelcoin_id.clone();

    let target = balance_id.clone();

    let mut balance_result = fuelcoin_instance
        .methods()
        .get_balance(asset_id.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    fuelcoin_instance
        .methods()
        .mint_coins(100)
        .call()
        .await
        .unwrap();

    balance_result = fuelcoin_instance
        .methods()
        .get_balance(asset_id.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 100);

    // confirm initial balance on balance contract (recipient)
    balance_result = fuelcoin_instance
        .methods()
        .get_balance(asset_id.clone(), target.clone())
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 0);

    let coins = 42u64;

    fuelcoin_instance
        .methods()
        .force_transfer_coins(coins, asset_id.clone(), target.clone())
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    // confirm remaining balance on fuelcoin contract
    balance_result = fuelcoin_instance
        .methods()
        .get_balance(asset_id.clone(), asset_id.clone())
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 58);

    // confirm new balance on balance contract (recipient)
    balance_result = fuelcoin_instance
        .methods()
        .get_balance(asset_id.clone(), target.clone())
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();
    assert_eq!(balance_result.value, 42);
}

#[tokio::test]
async fn can_mint_and_send_to_contract() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet.clone()).await;
    let balance_id = get_balance_contract_id(wallet).await;
    let amount = 55u64;

    let asset_id = fuelcoin_id.clone();

    let target = balance_id.clone();

    fuelcoin_instance
        .methods()
        .mint_and_send_to_contract(amount, target.clone())
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    let result = fuelcoin_instance
        .methods()
        .get_balance(asset_id, target)
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, amount)
}

#[tokio::test]
async fn can_mint_and_send_to_address() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet.clone()).await;
    let amount = 55u64;

    let asset_id_array: [u8; 32] = fuelcoin_id.into();

    let address = wallet.address();
    let recipient = address.clone();

    fuelcoin_instance
        .methods()
        .mint_and_send_to_address(amount, recipient.into())
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    assert_eq!(
        wallet
            .get_spendable_coins(AssetId::from(asset_id_array), 1)
            .await
            .unwrap()[0]
            .amount,
        amount.into()
    );
}

#[tokio::test]
async fn can_perform_generic_mint_to_with_address() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet.clone()).await;

    let amount = 55u64;
    let asset_id_array: [u8; 32] = fuelcoin_id.into();
    let address = wallet.address();

    fuelcoin_instance
        .methods()
        .generic_mint_to(amount, Identity::Address(address.into()))
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    assert_eq!(
        wallet
            .get_spendable_coins(AssetId::from(asset_id_array), 1)
            .await
            .unwrap()[0]
            .amount,
        amount.into()
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

    let wallets = launch_custom_provider_and_get_wallets(config, None, None).await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallets[0].clone()).await;
    let balance_id = get_balance_contract_id(wallets[0].clone()).await;
    let amount = 55u64;

    let target = balance_id.clone();

    fuelcoin_instance
        .methods()
        .generic_mint_to(amount, Identity::ContractId(target))
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    let result = fuelcoin_instance
        .methods()
        .get_balance(fuelcoin_id, target)
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, amount)
}

#[tokio::test]
async fn can_perform_generic_transfer_to_address() {
    let wallet = launch_provider_and_get_wallet().await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallet.clone()).await;

    let amount = 33u64;
    let asset_id_array: [u8; 32] = fuelcoin_id.into();
    let address = wallet.address();

    fuelcoin_instance
        .methods()
        .mint_coins(amount)
        .call()
        .await
        .unwrap();

    fuelcoin_instance
        .methods()
        .generic_transfer(amount, fuelcoin_id, Identity::Address(address.into()))
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    assert_eq!(
        wallet
            .get_spendable_coins(AssetId::from(asset_id_array), 1)
            .await
            .unwrap()[0]
            .amount,
        amount.into()
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
    let wallets = launch_custom_provider_and_get_wallets(config, None, None).await;

    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallets[0].clone()).await;
    let balance_id = get_balance_contract_id(wallets[0].clone()).await;

    let amount = 44u64;
    let to = balance_id.clone();

    fuelcoin_instance
        .methods()
        .mint_coins(amount)
        .call()
        .await
        .unwrap();

    fuelcoin_instance
        .methods()
        .generic_transfer(amount, fuelcoin_id, Identity::ContractId(to))
        .set_contracts(&[balance_id.into()])
        .call()
        .await
        .unwrap();

    let result = fuelcoin_instance
        .methods()
        .get_balance(fuelcoin_id, to)
        .set_contracts(&[balance_id.into()])
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

    let wallets = launch_custom_provider_and_get_wallets(config, None, None).await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallets[0].clone()).await;

    let amount = 33u64;
    let recipient_address: Address = wallets[0].address().into();

    let call_response = fuelcoin_instance
        .methods()
        .send_message(Bits256(*recipient_address), vec![100, 75, 50], amount)
        .append_message_outputs(1)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*fuelcoin_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(24, message_receipt.len().unwrap());
    assert_eq!(
        vec![0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 50],
        message_receipt.data().unwrap()
    );
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

    let wallets = launch_custom_provider_and_get_wallets(config, None, None).await;
    let (fuelcoin_instance, fuelcoin_id) = get_fuelcoin_instance(wallets[0].clone()).await;

    let amount = 33u64;
    let recipient_hex = "0x000000000000000000000000b46a7a1a23f3897cc83a94521a96da5c23bc58db";
    let recipient_address = Address::from_str(&recipient_hex).unwrap();

    let call_response = fuelcoin_instance
        .methods()
        .send_message(Bits256(*recipient_address), Vec::<u64>::new(), amount)
        .append_message_outputs(1)
        .call()
        .await
        .unwrap();

    let message_receipt = call_response
        .receipts
        .iter()
        .find(|&r| matches!(r, fuels::tx::Receipt::MessageOut { .. }))
        .unwrap();

    assert_eq!(*fuelcoin_id, **message_receipt.sender().unwrap());
    assert_eq!(&recipient_address, message_receipt.recipient().unwrap());
    assert_eq!(amount, message_receipt.amount().unwrap());
    assert_eq!(0, message_receipt.len().unwrap());
    assert_eq!(Vec::<u8>::new(), message_receipt.data().unwrap());
}

async fn get_fuelcoin_instance(wallet: WalletUnlocked) -> (TestFuelCoinContract, ContractId) {
    let fuelcoin_id = Contract::deploy(
        "test_projects/token_ops/out/debug/token_ops.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/token_ops/out/debug/token_ops-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let fuelcoin_instance = TestFuelCoinContract::new(fuelcoin_id.clone(), wallet);

    (fuelcoin_instance, fuelcoin_id.into())
}

async fn get_balance_contract_id(wallet: WalletUnlocked) -> ContractId {
    let balance_id = Contract::deploy(
        "test_artifacts/balance_contract/out/debug/balance_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/token_ops/out/debug/token_ops-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    balance_id.into()
}
