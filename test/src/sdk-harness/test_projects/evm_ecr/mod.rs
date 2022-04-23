use fuel_core::service::Config;
use fuel_tx::Transaction;
use fuels_contract::script::Script;
use fuels_signers::provider::Provider;
use std::fs::read;

async fn execute_script(bin_path: &str) -> u64 {
    let bin = read(bin_path);
    let client = Provider::launch(Config::local_node()).await.unwrap();

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: 1_000_000,
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

    receipts[0].val().unwrap()
}

#[tokio::test]
async fn evm_ecr_implementation() {
    let path_to_bin = "test_projects/evm_ecr/out/debug/evm_ecr.bin";
    let return_val = execute_script(path_to_bin).await;
    assert_eq!(1, return_val);
}