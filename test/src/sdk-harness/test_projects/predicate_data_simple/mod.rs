use fuel_vm::consts::*;
use fuel_vm::prelude::Opcode;
use fuels::contract::abi_encoder::ABIEncoder;
use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::signers::wallet::Wallet;
use fuels::test_helpers::Config;
use fuels::tx::{Address, AssetId, Contract, Input, Output, Transaction, UtxoId};
use std::str::FromStr;

async fn setup() -> (Vec<u8>, Address, Wallet, u64, AssetId) {
    let predicate_code =
        std::fs::read("test_projects/predicate_data_simple/out/debug/predicate_data_simple.bin")
            .unwrap();
    let predicate_address = (*Contract::root_from_code(&predicate_code)).into();

    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::default(),
        Some(Config {
            predicates: true,
            utxo_validation: true,
            ..Config::local_node()
        }),
    )
    .await;

    (
        predicate_code,
        predicate_address,
        wallets[0].clone(),
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
            wallet.get_asset_balance(&asset_id).await.unwrap(),
            0,
        )
        .await
        .unwrap();

    let output_coin = Output::coin(predicate_address, amount_to_predicate, asset_id);
    let output_change = Output::change(wallet.address().into(), 0, asset_id.into());
    let mut tx = Transaction::script(
        1,
        1000000,
        1,
        0,
        Opcode::RET(REG_ONE).to_bytes().to_vec(),
        vec![],
        wallet_coins,
        vec![output_coin, output_change],
        vec![],
    );
    wallet.sign_transaction(&mut tx).await.unwrap();
    wallet
        .get_provider()
        .unwrap()
        .send_transaction(&tx)
        .await
        .unwrap();
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
    let utxo_predicate_hash = wallet
        .get_provider()
        .unwrap()
        .get_spendable_coins(&predicate_address.into(), asset_id, amount_to_predicate)
        .await
        .unwrap();

    let mut inputs = vec![];
    let mut total_amount_in_predicate = 0;

    for coin in utxo_predicate_hash {
        let input_coin = Input::coin_predicate(
            UtxoId::from(coin.utxo_id),
            coin.owner.into(),
            coin.amount.0,
            asset_id,
            0,
            predicate_code.clone(),
            predicate_data.clone(),
        );
        inputs.push(input_coin);
        total_amount_in_predicate += coin.amount.0;
    }

    let output_coin = Output::coin(receiver_address, total_amount_in_predicate, asset_id);
    let output_change = Output::change(predicate_address, 0, asset_id);
    let new_tx = Transaction::script(
        0,
        1000000,
        0,
        0,
        vec![],
        vec![],
        inputs,
        vec![output_coin, output_change],
        vec![],
    );

    let script = Script::new(new_tx);
    let _call_result = script.call(&wallet.get_provider().unwrap()).await;
}

async fn get_balance(wallet: &Wallet, address: Address, asset_id: AssetId) -> u64 {
    wallet
        .get_provider()
        .unwrap()
        .get_asset_balance(&address.into(), asset_id)
        .await
        .unwrap()
}

#[tokio::test]
async fn valid_predicate_data_simple() {
    let arg = Token::U32(12345_u32);
    let args: Vec<Token> = vec![arg];
    let predicate_data = ABIEncoder::encode(&args).unwrap();

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
        receiver_balance_before + amount_to_predicate,
        receiver_balance_after
    );

    let predicate_balance = get_balance(&wallet, predicate_address, asset_id).await;
    assert_eq!(predicate_balance, 0);
}

#[tokio::test]
async fn invalid_predicate_data_simple() {
    let arg = Token::U32(1001_u32);
    let args: Vec<Token> = vec![arg];
    let predicate_data = ABIEncoder::encode(&args).unwrap();

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
    assert_eq!(predicate_balance, amount_to_predicate);
}
