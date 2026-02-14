use fuel_vm::fuel_tx::Contract as FuelsTxContract;
use fuels::{prelude::*, types::Bits256};

abigen!(Contract(
    name = "ContractBytecodeTest",
    abi = "out/contract_bytecode-abi.json"
));

#[tokio::test]
async fn can_get_bytecode_root() {
    let wallet = launch_provider_and_get_wallet().await.unwrap();

    let (contract_instance, id) = get_test_contract_instance(wallet).await;

    let bytecode_root = contract_instance
        .methods()
        .get_contract_bytecode_root(ContractId::from(id.clone()))
        .with_contracts(&[&contract_instance])
        .call()
        .await
        .unwrap()
        .value;

    let contract_bytecode =
        std::fs::read("out/contract_bytecode.bin").unwrap();
    let expected_bytecode_root = Bits256(*FuelsTxContract::root_from_code(contract_bytecode));

    assert_eq!(expected_bytecode_root, bytecode_root);
}

async fn get_test_contract_instance(
    wallet: Wallet,
) -> (ContractBytecodeTest<Wallet>, ContractId) {
    let id = Contract::load_from(
        "out/contract_bytecode.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap()
    .contract_id;

    let instance = ContractBytecodeTest::new(id.clone(), wallet);

    (instance, id)
}
