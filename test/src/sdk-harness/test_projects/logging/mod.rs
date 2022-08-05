use fuel_core::service::{Config, FuelService};
use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::tx::{ConsensusParameters, Transaction};
use hex;

#[tokio::test]
async fn run_valid() {
    let bin = std::fs::read("test_projects/logging/out/debug/logging.bin");

    let wallet = LocalWallet::new_random(None);
    let coins = setup_single_asset_coins(
        wallet.address(),
        BASE_ASSET_ID,
        1,
        1,
    );

    let (provider, _address) = setup_test_provider(coins.clone(), None).await;

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: ConsensusParameters::DEFAULT.max_gas_per_tx,
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
    let receipts = script.call(&provider).await.unwrap();

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert!(correct_hex.unwrap() == receipts[0].data().unwrap());
}
