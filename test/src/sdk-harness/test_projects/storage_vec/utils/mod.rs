use fuels::{prelude::*, tx::ContractId};
// Load abi from json
abigen!(MyContract, "test_artifacts/storage_vec/svec_u8/out/debug/svec_u8-abi.json");


pub mod setup {
    use super::*;

    pub async fn get_contract_instance() -> (MyContract, ContractId) {
        // Launch a local network and deploy the contract
        let wallet = launch_provider_and_get_single_wallet().await;
    
        let id = Contract::deploy("test_artifacts/storage_vec/svec_u8/out/debug/svec_u8.bin", &wallet, TxParameters::default())
            .await
            .unwrap();
    
        let instance = MyContract::new(id.to_string(), wallet);
    
        (instance, id)
    }
    
}

pub mod wrappers {
    use super::*;

    pub async fn push(instance: &MyContract, value: u8) {
        instance.vec_u8_push(value).call().await.unwrap();
    }
    
    pub async fn get(instance: &MyContract, index: u64) -> u8 {
        instance.vec_u8_get(index).call().await.unwrap().value
    }

    pub async fn pop(instance: &MyContract) -> u8 {
        instance.vec_u8_pop().call().await.unwrap().value
    }

    pub async fn remove(instance: &MyContract, index: u64) -> u8 {
        instance.vec_u8_remove(index).call().await.unwrap().value
    }

    pub async fn swap_remove(instance: &MyContract, index: u64) -> u8 {
        instance.vec_u8_swap_remove(index).call().await.unwrap().value
    }
}
