use assert_matches::assert_matches;
use fuels::{prelude::*, tx::Receipt, types::transaction_builders::ScriptTransactionBuilder};

async fn call_script(script_data: Vec<u8>) -> Result<Vec<Receipt>> {
    let wallet = launch_provider_and_get_wallet().await.unwrap();
    let provider = wallet.provider().unwrap();

    let wallet_coins = wallet
        .get_asset_inputs_for_amount(
            AssetId::default(),
            wallet.get_asset_balance(&AssetId::default()).await.unwrap(),
        )
        .await
        .unwrap();

    let mut tx =
        ScriptTransactionBuilder::prepare_transfer(wallet_coins, vec![], Default::default())
            .with_script(std::fs::read(
                "test_projects/script_data/out/release/script_data.bin",
            )?)
            .with_script_data(script_data);

    tx.add_signer(wallet.clone()).unwrap();

    let tx = tx.build(provider).await?;

    let provider = wallet.provider().unwrap();
    let tx_id = provider.send_transaction(tx).await.unwrap();
    provider
        .tx_status(&tx_id)
        .await
        .unwrap()
        .take_receipts_checked(None)
}

#[tokio::test]
async fn script_data() {
    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a").unwrap();
    let call_result = call_script(correct_hex.clone()).await;
    assert_matches!(call_result, Ok(_));
    let receipts = call_result.unwrap();
    assert_eq!(correct_hex, receipts[0].data().unwrap());

    let bad_hex =
        hex::decode("bad6afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a").unwrap();
    let call_result = call_script(bad_hex).await;
    assert_matches!(call_result, Err(_));
}
