use assert_matches::assert_matches;
use fuels::{
    prelude::*,
    tx::Receipt,
    types::transaction_builders::{ScriptTransactionBuilder, TransactionBuilder},
};

async fn call_script(script_data: Vec<u8>) -> Result<Vec<Receipt>> {
    let wallet = launch_provider_and_get_wallet().await;

    let mut tx =
        ScriptTransactionBuilder::prepare_transfer(vec![], vec![], TxParameters::default())
            .set_script(std::fs::read(
                "test_projects/script_data/out/debug/script_data.bin",
            )?)
            .set_script_data(script_data)
            .build()?;

    let params = wallet
        .provider()
        .unwrap()
        .consensus_parameters()
        .await
        .unwrap();
    wallet.sign_transaction(&mut tx, &params)?;

    wallet.provider().unwrap().send_transaction(&tx).await
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
