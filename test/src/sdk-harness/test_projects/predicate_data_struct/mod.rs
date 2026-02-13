use fuel_vm::fuel_asm::{op, RegId};
use fuel_vm::fuel_tx;
use fuel_vm::fuel_tx::{Address, AssetId, Output};
use fuels::{
    core::codec::{ABIEncoder, EncoderConfig},
    prelude::*,
    types::{input::Input, transaction_builders::ScriptTransactionBuilder, Token},
};
use std::str::FromStr;

async fn setup() -> (Vec<u8>, Address, Wallet, u64, AssetId) {
    let predicate_code =
        std::fs::read("out_for_sdk_harness_tests/predicate_data_struct.bin")
            .unwrap();
    let predicate_address = fuel_tx::Input::predicate_owner(&predicate_code);

    let mut node_config = NodeConfig::default();
    node_config.starting_gas_price = 0;
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(Some(1), None, None),
        Some(node_config),
        None,
    )
    .await
    .unwrap();
    let wallet = wallets.pop().unwrap();
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
    wallet: &Wallet,
    amount_to_predicate: u64,
    asset_id: AssetId,
) {
    let wallet_coins = wallet
        .get_asset_inputs_for_amount(
            asset_id,
            wallet.get_asset_balance(&asset_id).await.unwrap().into(),
            None,
        )
        .await
        .unwrap();

    let provider = wallet.provider();
    let output_coin = Output::coin(predicate_address, amount_to_predicate, asset_id);
    let output_change = Output::change(wallet.address().into(), 0, asset_id);
    let mut tx = ScriptTransactionBuilder::prepare_transfer(
        wallet_coins,
        vec![output_coin, output_change],
        Default::default(),
    )
    .with_script(op::ret(RegId::ONE).to_bytes().to_vec());

    tx.add_signer(wallet.signer().clone()).unwrap();
    let tx = tx.build(provider).await.unwrap();
    provider.send_transaction_and_await_commit(tx).await.unwrap();
}

async fn submit_to_predicate(
    predicate_code: Vec<u8>,
    predicate_address: Address,
    wallet: &Wallet,
    amount_to_predicate: u64,
    asset_id: AssetId,
    receiver_address: Address,
    predicate_data: Vec<u8>,
) {
    let filter = ResourceFilter {
        from: predicate_address.into(),
        asset_id: Some(asset_id),
        amount: amount_to_predicate.into(),
        ..Default::default()
    };
    let provider = wallet.provider();

    let utxo_predicate_hash = provider.get_spendable_resources(filter).await.unwrap();

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

    let output_coin = Output::coin(receiver_address, total_amount_in_predicate, asset_id);
    let output_change = Output::change(predicate_address, 0, asset_id);

    let new_tx = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        vec![output_coin, output_change],
        Default::default(),
    )
    .build(provider)
    .await
    .unwrap();

    let _call_result = provider.send_transaction_and_await_commit(new_tx).await;
}

async fn get_balance(wallet: &Wallet, address: Address, asset_id: AssetId) -> u128 {
    wallet
        .provider()
        .get_asset_balance(&address.into(), &asset_id)
        .await
        .unwrap()
}

struct Validation {
    has_account: bool,
    total_complete: u64,
}

fn encode_struct(predicate_struct: Validation) -> Vec<u8> {
    let has_account = Token::Bool(predicate_struct.has_account);
    let total_complete = Token::U64(predicate_struct.total_complete);
    let token_struct: Vec<Token> = vec![has_account, total_complete];
    ABIEncoder::new(EncoderConfig::default())
        .encode(&token_struct)
        .unwrap()
}

#[tokio::test]
async fn should_pass_with_valid_struct() {
    let predicate_data = encode_struct(Validation {
        has_account: true,
        total_complete: 100,
    });
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
    .await;

    let receiver_balance_after = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(
        receiver_balance_before + amount_to_predicate as u128,
        receiver_balance_after
    );

    let predicate_balance = get_balance(&wallet, predicate_address, asset_id).await;
    assert_eq!(predicate_balance, 0);
}

#[tokio::test]
async fn should_fail_with_invalid_struct_u64() {
    let predicate_data = encode_struct(Validation {
        has_account: true,
        total_complete: 200,
    });
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
    .await;

    let receiver_balance_after = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(receiver_balance_before, receiver_balance_after);

    let predicate_balance = get_balance(&wallet, predicate_address, asset_id).await;
    assert_eq!(predicate_balance, amount_to_predicate as u128);
}

#[tokio::test]
async fn should_fail_with_invalid_struct_bool() {
    let predicate_data = encode_struct(Validation {
        has_account: false,
        total_complete: 100,
    });
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
    .await;

    let receiver_balance_after = get_balance(&wallet, receiver_address, asset_id).await;
    assert_eq!(receiver_balance_before, receiver_balance_after);

    let predicate_balance = get_balance(&wallet, predicate_address, asset_id).await;
    assert_eq!(predicate_balance, amount_to_predicate as u128);
}
