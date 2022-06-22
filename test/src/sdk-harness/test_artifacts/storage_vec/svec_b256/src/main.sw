contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_b256_push(value: b256);
    #[storage(read)]
    fn vec_b256_get(index: u64) -> Option<b256>;
    #[storage(read, write)]
    fn vec_b256_pop() -> Option<b256>;
    #[storage(read, write)]
    fn vec_b256_remove(index: u64) -> Result<b256, StorageVecError>;
    #[storage(read, write)]
    fn vec_b256_swap_remove(index: u64) -> Result<b256, StorageVecError>;
    #[storage(read, write)]
    fn vec_b256_insert(index: u64, value: b256) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_b256_len() -> u64;
    #[storage(read)]
    fn vec_b256_is_empty() -> bool;
    #[storage(write)]
    fn vec_b256_clear();
}

storage {
    vec_b256: StorageVec<b256>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_b256_push(value: b256) {
        storage.vec_b256.push(value);
    }
    #[storage(read)]
    fn vec_b256_get(index: u64) -> Option<b256> {
        storage.vec_b256.get(index)
    }
    #[storage(read, write)]
    fn vec_b256_pop() -> Option<b256> {
        storage.vec_b256.pop()
    }
    #[storage(read, write)]
    fn vec_b256_remove(index: u64) -> Result<b256, StorageVecError> {
        let res: Result<b256, StorageVecError> = storage.vec_b256.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_b256_swap_remove(index: u64) -> Result<b256, StorageVecError> {
        storage.vec_b256.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_b256_insert(index: u64, value: b256) -> Result<(), StorageVecError> {
        storage.vec_b256.insert(index, value)
    }
    #[storage(read)]
    fn vec_b256_len() -> u64 {
        storage.vec_b256.len()
    }
    #[storage(read)]
    fn vec_b256_is_empty() -> bool {
        storage.vec_b256.is_empty()
    }
    #[storage(write)]
    fn vec_b256_clear() {
        storage.vec_b256.clear();
    }
}