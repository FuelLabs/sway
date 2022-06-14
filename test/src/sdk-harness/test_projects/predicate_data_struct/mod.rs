use assert_matches::assert_matches;
use fuel_core::service::{Config, FuelService};
use fuel_crypto::SecretKey;
use fuel_gql_client::client::FuelClient;
use fuel_vm::consts::*;
use fuel_vm::prelude::Opcode;
use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::signers::wallet::*;
use fuels::contract::abi_encoder::ABIEncoder;
use fuels::tx::{Address, AssetId, Contract, Input, Output, Receipt, Transaction, UtxoId};
use std::str::FromStr;

#[tokio::test]
async fn valid_predicate_data_struct() {
    let amount_to_predicate = 1000;
    let asset_id = AssetId::default();
    let predicate_code = std::fs::read("test_projects/predicate_data_struct/out/debug/predicate_data_struct.bin");
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let secret_key = SecretKey::from_str("0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb")
        .unwrap();

    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;
    let server = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(server.bound_address);
    let provider = Provider::new(client.clone());
    let mut wallet = Wallet::new_from_private_key(secret_key, None);
    wallet.set_provider(provider.clone());

    let amount = wallet.get_asset_balance(&asset_id).await.unwrap();
    let wallet_coins = wallet
        .get_asset_inputs_for_amount(asset_id, amount, 0)
        .await
        .unwrap();

    let output_coin = Output::coin(predicate_hash, amount_to_predicate, asset_id);
    let mut output_change = Output::change(wallet.address(), 0, asset_id);
    let script = Opcode::RET(REG_ONE).to_bytes().to_vec();
    let mut tx = Transaction::script(
        1,
        1000000,
        1,
        0,
        script.clone(),
        vec![],
        wallet_coins,
        vec![output_coin, output_change],
        vec![],
    );
    let signature = wallet.sign_transaction(&mut tx).await.unwrap();
    let mut tx_receipts = provider.send_transaction(&tx).await.unwrap();

    // SPEND
    let has_account = Token::Bool(true);
    let total_complete = Token::U64(100);

    // Create the custom struct token using the array of tuples above
    let arg = Token::Struct(vec![has_account, total_complete]);
    let args: Vec<Token> = vec![arg];
    let predicate_data = encoder.encode(&args).unwrap();
    let utxo_predicate_hash = provider
        .get_spendable_coins(&predicate_hash, asset_id, amount_to_predicate)
        .await
        .unwrap();

    let mut inputs = vec![];
    let mut tot_amount = 0;

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
        tot_amount += coin.amount.0;
    }
    let receiver_address =
        Address::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c")
            .unwrap();
    let receiver_balance_before = provider
        .get_asset_balance(&receiver_address, asset_id)
        .await
        .unwrap();
    assert_eq!(receiver_balance_before, 0);
    let output_coin = Output::coin(receiver_address, tot_amount, asset_id);

    let output_change = Output::change(predicate_hash, 0, asset_id);

    let mut new_tx = Transaction::script(
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
    let call_result = script.call(&client).await;

    let receiver_balance_after = provider
        .get_asset_balance(&receiver_address, asset_id)
        .await
        .unwrap();
    assert_eq!(
        receiver_balance_before + amount_to_predicate,
        receiver_balance_after
    );

    let predicate_balance = provider
        .get_asset_balance(&predicate_hash, asset_id)
        .await
        .unwrap();
    assert_eq!(predicate_balance, 0);
}
