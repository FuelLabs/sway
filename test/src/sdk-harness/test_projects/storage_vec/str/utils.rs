use fuels::{prelude::*, tx::ContractId};
// Load abi from json
abigen!(
    MyContract,
    "test_artifacts/storage_vec/svec_str/out/debug/svec_str-abi.json"
);

pub mod setup {
    use super::*;

    pub async fn get_contract_instance() -> (MyContract, ContractId) {
        // Launch a local network and deploy the contract
        let wallet = launch_provider_and_get_wallet().await;

        let id = Contract::deploy(
            "test_artifacts/storage_vec/svec_str/out/debug/svec_str.bin",
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                "test_artifacts/storage_vec/svec_str/out/debug/svec_str-storage_slots.json"
                    .to_string(),
            )),
        )
        .await
        .unwrap();

        let instance = MyContractBuilder::new(id.to_string(), wallet).build();

        (instance, id)
    }
}

pub mod wrappers {
    use super::*;

    pub async fn push(instance: &MyContract, value: String) {
        instance.str_push(value).call().await.unwrap();
    }

    pub async fn get(instance: &MyContract, index: u64) -> String {
        instance.str_get(index).call().await.unwrap().value
    }

    pub async fn pop(instance: &MyContract) -> String {
        instance.str_pop().call().await.unwrap().value
    }

    pub async fn remove(instance: &MyContract, index: u64) -> String {
        instance.str_remove(index).call().await.unwrap().value
    }

    pub async fn swap_remove(instance: &MyContract, index: u64) -> String {
        instance.str_swap_remove(index).call().await.unwrap().value
    }

    pub async fn set(instance: &MyContract, index: u64, value: String) {
        instance.str_set(index, value).call().await.unwrap();
    }

    pub async fn insert(instance: &MyContract, index: u64, value: String) {
        instance.str_insert(index, value).call().await.unwrap();
    }

    pub async fn len(instance: &MyContract) -> u64 {
        instance.str_len().call().await.unwrap().value
    }

    pub async fn is_empty(instance: &MyContract) -> bool {
        instance.str_is_empty().call().await.unwrap().value
    }

    pub async fn clear(instance: &MyContract) {
        instance.str_clear().call().await.unwrap();
    }
}
