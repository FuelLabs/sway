use fuels::prelude::*;
use fuels::tx::{ContractId};

abigen!(
    TestBytecodeContract,
    "test_projects/contract_bytecode/out/debug/contract_bytecode-abi.json"
);


abigen!(
    BytecodeContract,
    "test_artifacts/contract_bytecode/out/debug/contract_bytecode-abi.json"
);

// This is the contract which will fetch the external bytecode root
async fn get_test_contract_instance(wallet: WalletUnlocked) -> (TestBytecodeContract, ContractId) {
    let id = Contract::deploy(
        "test_projects/contract_bytecode/out/debug/contract_bytecode.bin",
        &wallet,
        TxParameters::default(),
        )
    .await
    .unwrap();

    let instance = TestBytecodeContract::new(id.to_string(), wallet);

    (instance, id.into())
}

// This is the (artifact) contract whose bytecode root will be read
async fn get_bytecode_contract_instance(wallet: WalletUnlocked) -> (BytecodeContract, ContractId) {
    let id = Contract::deploy(
        "test_artifacts/contract_bytecode/out/debug/contract_bytecode.bin",
        &wallet,
        TxParameters::default(),
        )
    .await
    .unwrap();

    let instance = BytecodeContract::new(id.to_string(), wallet);

    (instance, id.into())
}



#[tokio::test]
async fn can_get_bytecode_root() {
    let wallet = launch_provider_and_get_wallet().await;

    let (contract_instance, _) = get_bytecode_contract_instance(wallet).await;
    let (_, id) = get_test_contract_instance(wallet).await;


    let mut root = contract_instance
        .methods()
        .get_contract_bytecode_root(id)
        .call()
        .await
        .unwrap();


    println!("{:?}", root);

}