use fuels::{prelude::*, tx::ContractId};
// Load abi from json
abigen!(
    MyContract,
    "test_artifacts/storage_vec/svec_array/out/debug/svec_array-abi.json"
);

pub mod setup {
    use super::*;

    pub async fn get_contract_instance() -> (MyContract, ContractId) {
        // Launch a local network and deploy the contract
        let wallet = launch_provider_and_get_wallet().await;

        let id = Contract::deploy(
            "test_artifacts/storage_vec/svec_array/out/debug/svec_array.bin",
            &wallet,
            TxParameters::default(),
            StorageConfiguration::with_storage_path(Some(
                "test_artifacts/storage_vec/svec_array/out/debug/svec_array-storage_slots.json"
                    .to_string(),
            )),
        )
        .await
        .unwrap();

        let instance = MyContract::new(id.to_string(), wallet);

        (instance, id.into())
    }
}

pub mod wrappers {
    use super::*;

    pub async fn push(instance: &MyContract, value: [u8; 3]) {
        instance.methods().array_push(value).call().await.unwrap();
    }

    pub async fn get(instance: &MyContract, index: u64) -> [u8; 3] {
        instance
            .methods()
            .array_get(index)
            .call()
            .await
            .unwrap()
            .value
    }

    pub async fn pop(instance: &MyContract) -> [u8; 3] {
        instance.methods().array_pop().call().await.unwrap().value
    }

    pub async fn remove(instance: &MyContract, index: u64) -> [u8; 3] {
        instance
            .methods()
            .array_remove(index)
            .call()
            .await
            .unwrap()
            .value
    }

    pub async fn swap_remove(instance: &MyContract, index: u64) -> [u8; 3] {
        instance
            .methods()
            .array_swap_remove(index)
            .call()
            .await
            .unwrap()
            .value
    }

    pub async fn set(instance: &MyContract, index: u64, value: [u8; 3]) {
        instance
            .methods()
            .array_set(index, value)
            .call()
            .await
            .unwrap();
    }

    pub async fn insert(instance: &MyContract, index: u64, value: [u8; 3]) {
        instance
            .methods()
            .array_insert(index, value)
            .call()
            .await
            .unwrap();
    }

    pub async fn len(instance: &MyContract) -> u64 {
        instance.methods().array_len().call().await.unwrap().value
    }

    pub async fn is_empty(instance: &MyContract) -> bool {
        instance
            .methods()
            .array_is_empty()
            .call()
            .await
            .unwrap()
            .value
    }

    pub async fn clear(instance: &MyContract) {
        instance.methods().array_clear().call().await.unwrap();
    }
}
