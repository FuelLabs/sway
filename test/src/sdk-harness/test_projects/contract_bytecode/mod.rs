use fuels::{prelude::*, tx::Contract as FuelsTxContract, types::Bits256};

abigen!(Contract(
    name = "ContractBytecodeTest",
    abi = "test_projects/contract_bytecode/out/debug/contract_bytecode-abi.json"
));

#[tokio::test]
async fn can_get_bytecode_root() {
    let wallet = launch_provider_and_get_wallet().await;

    let (contract_instance, id) = get_test_contract_instance(wallet).await;

    let bytecode_root = contract_instance
        .methods()
        .get_contract_bytecode_root(ContractId::from(id.clone()))
        .set_contracts(&[&contract_instance])
        .call()
        .await
        .unwrap()
        .value;

    let contract_bytecode =
        std::fs::read("test_projects/contract_bytecode/out/debug/contract_bytecode.bin").unwrap();
    let expected_bytecode_root = Bits256(*FuelsTxContract::root_from_code(&contract_bytecode));

    assert_eq!(expected_bytecode_root, bytecode_root);
}

async fn get_test_contract_instance(
    wallet: WalletUnlocked,
) -> (ContractBytecodeTest<WalletUnlocked>, Bech32ContractId) {
    let id = Contract::deploy(
        "test_projects/contract_bytecode/out/debug/contract_bytecode.bin",
        &wallet,
        DeployConfiguration::default(),
    )
    .await
    .unwrap();

    let instance = ContractBytecodeTest::new(id.clone(), wallet);

    (instance, id)
}
