use assert_matches::assert_matches;
use fuel_core::service::{Config, FuelService};
use fuel_gql_client::client::FuelClient;
use fuels::contract::script::Script;
use fuels::tx::{ConsensusParameters, Receipt, Transaction};

async fn call_script(script_data: Vec<u8>) -> Result<Vec<Receipt>, fuels::prelude::Error> {
    let bin = std::fs::read("test_projects/script_data/out/debug/script_data.bin");
    let server = FuelService::new_node(Config::local_node()).await.unwrap();
    let client = FuelClient::from(server.bound_address);

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: ConsensusParameters::DEFAULT.max_gas_per_tx,
        maturity: 0,
        byte_price: 0,
        receipts_root: Default::default(),
        script: bin.unwrap(),
        script_data,
        inputs: vec![],
        outputs: vec![],
        witnesses: vec![vec![].into()],
        metadata: None,
    };

    let script = Script::new(tx);
    script.call(&client).await
}

#[tokio::test]
async fn script_data() {
    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a").unwrap();
    let call_result = call_script(correct_hex.clone()).await;
    assert_matches!(call_result, Ok(_));
    let receipts = call_result.unwrap();
    assert!(correct_hex == receipts[0].data().unwrap());

    let bad_hex =
        hex::decode("bad6afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a").unwrap();
    let call_result = call_script(bad_hex).await;
    assert_matches!(call_result, Err(_));
}
