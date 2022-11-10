use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::tx::{ConsensusParameters, Transaction};
use hex;

#[tokio::test]
async fn run_valid() {
    let bin = std::fs::read("test_projects/logging/out/debug/logging.bin");

    let wallet = launch_provider_and_get_wallet().await;

    let mut tx = Transaction::script(
        0,
        ConsensusParameters::DEFAULT.max_gas_per_tx,
        0,
        bin.unwrap(),
        vec![],
        vec![],
        vec![],
        vec![],
    );

    wallet.sign_transaction(&mut tx).await.unwrap();

    let provider = wallet.get_provider().unwrap();

    let receipts = Script::new(tx).call(&provider).await.unwrap();

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert_eq!(correct_hex.unwrap(), receipts[0].data().unwrap());
}
