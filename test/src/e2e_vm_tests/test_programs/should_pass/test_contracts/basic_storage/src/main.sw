contract;
use std::storage::*;
use basic_storage_abi::*;

impl StoreU64 for Contract {
    fn get_u64(storage_key: b256) -> u64 {
        get(storage_key)
    }

    fn store_u64(key: b256, value: u64) {
        store(key, value);
    }
}
