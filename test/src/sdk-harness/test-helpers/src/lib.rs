//! util & helper functions to support working with the Rust SDK (fuels-rs)

use fuel_core::service::Config;
use fuel_tx::{consts::MAX_GAS_PER_TX, Transaction};
use fuels::contract::script::Script;
use fuels::prelude::*;
use std::fs::read;

/// Helper function to reduce boilerplate code in tests.
/// Used to run a script which returns a boolean value.
pub async fn script_runner(bin_path: &str) -> u64 {
    let bin = read(bin_path);
    let client = Provider::launch(Config::local_node()).await.unwrap();

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

    receipts[0].val().unwrap()
}
