use fuel_core::service::Config;
use fuel_tx::{consts::MAX_GAS_PER_TX, Transaction};
use fuels::contract::script::Script;
use fuels::prelude::*;
use hex;

#[tokio::test]
async fn run_valid() {
    let bin = std::fs::read("test_projects/logging/out/debug/logging.bin");
    let client = Provider::launch(Config::local_node()).await.unwrap();

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: MAX_GAS_PER_TX,
        maturity: 0,
        byte_price: 0,
        receipts_root: Default::default(),
        script: bin.unwrap(),
        script_data: vec![],
        inputs: vec![],
        outputs: vec![],
        witnesses: vec![vec![].into()],
        metadata: None,
    };

    let script = Script::new(tx);
    let receipts = script.call(&client).await.unwrap();

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert!(correct_hex.unwrap() == receipts[0].data().unwrap());
}
