use fuel_core::service::{Config, FuelService};
use fuel_gql_client::client::FuelClient;
use fuels::contract::script::Script;
use fuels::tx::{ConsensusParameters, Transaction};
use hex;

#[tokio::test]
async fn run_valid() {
    let bin = std::fs::read("test_projects/logging/out/debug/logging.bin");
    let server = FuelService::new_node(Config::local_node()).await.unwrap();
    let client = FuelClient::from(server.bound_address);

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
    let receipts = script.call(&client).await.unwrap();

    let correct_hex =
        hex::decode("ef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a");

    assert!(correct_hex.unwrap() == receipts[0].data().unwrap());
}
