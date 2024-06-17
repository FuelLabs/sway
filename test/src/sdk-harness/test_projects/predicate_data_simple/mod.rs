use fuel_vm::fuel_asm::{op, RegId};
use fuel_vm::fuel_tx;
use fuel_vm::fuel_tx::{Address, AssetId, Output};
use fuels::{
    accounts::wallet::{Wallet, WalletUnlocked},
    core::codec::{ABIEncoder, EncoderConfig},
    prelude::*,
    types::{input::Input, transaction_builders::ScriptTransactionBuilder, Token},
};
use std::str::FromStr;

async fn setup() -> (Vec<u8>, Address, WalletUnlocked, u64, AssetId) {
    let predicate_code =
        std::fs::read("test_projects/predicate_data_simple/out/release/predicate_data_simple.bin")
            .unwrap();
    let predicate_address = fuel_tx::Input::predicate_owner(&predicate_code);

    let wallet = launch_provider_and_get_wallet().await.unwrap();
    (
        predicate_code,
        predicate_address,
        wallet,
        1000,
        AssetId::default(),
    )
}

async fn create_predicate(
    predicate_address: Address,
    wallet: &WalletUnlocked,
    amount_to_predicate: u64,
    asset_id: AssetId,
) {
    let provider = wallet.provider().unwrap();
    let wallet_coins = wallet
        .get_asset_inputs_for_amount(asset_id, wallet.get_asset_balance(&asset_id).await.unwrap())
        .await
        .unwrap();

    let output_coin = Output::coin(predicate_address, amount_to_predicate, asset_id);
    let output_change = Output::change(wallet.address().into(), 0, asset_id);

    let mut tx = ScriptTransactionBuilder::prepare_transfer(
        wallet_coins,
        vec![output_coin, output_change],
        Default::default(),
    )
    .with_script(op::ret(RegId::ONE).to_bytes().to_vec());

    tx.add_signer(wallet.clone()).unwrap();
    let tx = tx.build(provider).await.unwrap();

    provider.send_transaction(tx).await.unwrap();
}

async fn submit_to_predicate(
    predicate_code: Vec<u8>,
    predicate_address: Address,
    wallet: &Wallet,
    amount_to_predicate: u64,
    asset_id: AssetId,
    receiver_address: Address,
    predicate_data: Vec<u8>,
) -> Result<()> {
    let filter = ResourceFilter {
        from: predicate_address.into(),
        asset_id: Some(asset_id),
        amount: amount_to_predicate,
        ..Default::default()
    };

    let utxo_predicate_hash = wallet
        .provider()
        .unwrap()
        .get_spendable_resources(filter)
        .await
        .unwrap();

    let mut inputs = vec![];
    let mut total_amount_in_predicate = 0;

    for coin in utxo_predicate_hash {
        inputs.push(Input::resource_predicate(
            coin.clone(),
            predicate_code.to_vec(),
            predicate_data.clone(),
        ));
        total_amount_in_predicate += coin.amount();
    }

    let output_coin = Output::coin(receiver_address, total_amount_in_predicate - 1, asset_id);
    let output_change = Output::change(predicate_address, 0, asset_id);

    let provider = wallet.provider().unwrap();
    let new_tx = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        vec![output_coin, output_change],
        Default::default(),
    )
    .with_tx_policies(TxPolicies::default().with_tip(1))
    .build(provider)
    .await
    .unwrap();

    wallet
        .provider()
        .unwrap()
        .send_transaction(new_tx)
        .await
        .map(|_| ())
}

async fn get_balance(wallet: &Wallet, address: Address, asset_id: AssetId) -> u64 {
    wallet
        .provider()
        .unwrap()
        .get_asset_balance(&address.into(), asset_id)
        .await
        .unwrap()
}

#[tokio::test]
async fn valid_predicate_data_simple() {
    let arg = Token::U32(12345_u32);
    let args: Vec<Token> = vec![arg];
    let predicate_data = ABIEncoder::new(EncoderConfig::default())
        .encode(&args)
        .unwrap();

    let receiver_address =
        Address::from_str("0xd926978a28a565531a06cbf5fab5402d6ee2021e5a5dce2d2f7c61e5521be109")
            .unwrap();
    let (predicate_code, predicate_address, wallet, amount_to_predicate, asset_id) = setup().await;

    create_predicate(predicate_address, &wallet, amount_to_predicate, asset_id).await;

    let receiver_balance_before = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(receiver_balance_before, 0);

    submit_to_predicate(
        predicate_code,
        predicate_address,
        &wallet,
        amount_to_predicate,
        asset_id,
        receiver_address,
        predicate_data,
    )
    .await
    .expect("Failed to submit to predicate");

    let receiver_balance_after = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(
        receiver_balance_before + amount_to_predicate - 1,
        receiver_balance_after
    );

    let predicate_balance = get_balance(&wallet, predicate_address, asset_id).await;
    assert_eq!(predicate_balance, 0);
}

#[tokio::test]
async fn invalid_predicate_data_simple() {
    let arg = Token::U32(1001_u32);
    let args: Vec<Token> = vec![arg];
    let predicate_data = ABIEncoder::new(EncoderConfig::default())
        .encode(&args)
        .unwrap();

    let receiver_address =
        Address::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c")
            .unwrap();
    let (predicate_code, predicate_address, wallet, amount_to_predicate, asset_id) = setup().await;

    create_predicate(predicate_address, &wallet, amount_to_predicate, asset_id).await;

    let receiver_balance_before = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(receiver_balance_before, 0);

    submit_to_predicate(
        predicate_code,
        predicate_address,
        &wallet,
        amount_to_predicate,
        asset_id,
        receiver_address,
        predicate_data,
    )
    .await
    .expect_err("Submitting to predicate should have failed");

    let receiver_balance_after = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(receiver_balance_before, receiver_balance_after);

    let predicate_balance = get_balance(&wallet, predicate_address, asset_id).await;
    assert_eq!(predicate_balance, amount_to_predicate);
}
