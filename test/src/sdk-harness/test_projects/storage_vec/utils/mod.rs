use fuels::{prelude::*, tx::ContractId};
// Load abi from json
abigen!(MyContract, "test_artifacts/storage_vec/svec_u8/out/debug/svec_u8-abi.json");

pub async fn get_contract_instance() -> (MyContract, ContractId) {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_single_wallet().await;

    let id = Contract::deploy("test_artifacts/storage_vec/svec_u8/out/debug/svec_u8.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), wallet);

    (instance, id)
}

pub async fn push(instance: &MyContract, value: u8) {
    instance.vec_u8_push(value).call().await.unwrap();
}

pub async fn get(instance: &MyContract, index: u64) -> u8 {
    instance.vec_u8_get(index).call().await.unwrap().value
}