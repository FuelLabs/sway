use fuel_core::service::Config;
use fuel_tx::{Receipt, Transaction};
use fuel_types::ContractId;
use fuels_contract::script::Script;
use fuels_signers::provider::Provider;
use std::fs::read;

#[tokio::test]
async fn contract_id_eq_implementation() {
    let bin = read("test_projects/contract_id_type/out/debug/contract_id_type.bin");
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

    let expected_receipt = Receipt::Return {
        id: ContractId::new([0u8; 32]),
        val: 1,
        pc: receipts[0].pc().unwrap(),
        is: 10352,
    };

    assert_eq!(expected_receipt, receipts[0]);
}
