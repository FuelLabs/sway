use fuel_core::service::{Config, FuelService};
use fuel_gql_client::client::FuelClient;
use fuel_tx::{consts::MAX_GAS_PER_TX, Receipt, Transaction};
use fuels::contract::script::Script;

#[tokio::test]
async fn run_valid() {
    let bin = std::fs::read("test_projects/option/out/debug/option.bin");
    let server = FuelService::new_node(Config::local_node()).await.unwrap();
    let client = FuelClient::from(server.bound_address);

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: MAX_GAS_PER_TX,
        maturity: 0,
        byte_price: 0,
        receipts_root: Default::default(),
        script: bin.unwrap(), // Here we pass the compiled script into the transaction
        script_data: vec![],
        inputs: vec![],
        outputs: vec![],
        witnesses: vec![vec![].into()],
        metadata: None,
    };

    let script = Script::new(tx);
    let receipts = script.call(&client).await.unwrap();

    if let Receipt::Return { .. } = receipts[0] {
    } else {
        assert!(false);
    }
}
