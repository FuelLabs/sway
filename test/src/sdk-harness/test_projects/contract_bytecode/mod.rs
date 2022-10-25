use fuels::prelude::*;
use fuels::tx::Contract as FuelsTxContract;

abigen!(
    ContractBytecodeTest,
    "test_projects/contract_bytecode/out/debug/contract_bytecode-abi.json"
);

#[tokio::test]
async fn can_get_bytecode_root() {
    let wallet = launch_provider_and_get_wallet().await;

    let (contract_instance, id) = get_test_contract_instance(wallet).await;

    let bytecode_root = contract_instance
        .methods()
        .get_contract_bytecode_root(ContractId::from(id.clone()))
        .set_contracts(&[id.clone()])
        .call()
        .await
        .unwrap()
        .value;

    let contract_bytecode =
        std::fs::read("test_projects/contract_bytecode/out/debug/contract_bytecode.bin").unwrap();
    let expected_bytecode_root = Bits256(*FuelsTxContract::root_from_code(&contract_bytecode));

    assert_eq!(expected_bytecode_root, bytecode_root);
}

#[tokio::test]
async fn can_get_bytecode_size() {
    let wallet = launch_provider_and_get_wallet().await;

    let (contract_instance, id) = get_test_contract_instance(wallet).await;

    let bytecode_size = contract_instance
        .methods()
        .get_contract_bytecode_size(ContractId::from(id.clone()))
        .set_contracts(&[id.clone()])
        .call()
        .await
        .unwrap()
        .value;


    let contract_bytecode =
    std::fs::read("test_projects/contract_bytecode/out/debug/contract_bytecode.bin").unwrap();
    let expected_size : u64 = contract_bytecode.len().try_into().unwrap();
    
    assert_eq!(expected_size, bytecode_size);
}

#[tokio::test]
async fn can_get_b256_from_bytecode() {
  
    let wallet = launch_provider_and_get_wallet().await;
    let (contract_instance, id) = get_test_contract_instance(wallet).await;

    let index : usize = 12;

    let result = contract_instance
        .methods()
        .get_b256_from_bytecode(index as u64, ContractId::from(id.clone()))
        .set_contracts(&[id.clone()])
        .call()
        .await
        .unwrap()
        .value
        .0;

    let contract_bytecode =
    std::fs::read("test_projects/contract_bytecode/out/debug/contract_bytecode.bin").unwrap();

    let slice : &[u8] = &contract_bytecode[index..(index+32)];
    let arr : [u8; 32] = slice.try_into().unwrap();

    assert_eq!(result, arr);
}


async fn get_test_contract_instance(
    wallet: WalletUnlocked,
) -> (ContractBytecodeTest, Bech32ContractId) {
    let id = Contract::deploy(
        "test_projects/contract_bytecode/out/debug/contract_bytecode.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "test_projects/contract_bytecode/out/debug/contract_bytecode-storage_slots.json"
                .to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = ContractBytecodeTest::new(id.to_string(), wallet);

    (instance, id)
}
