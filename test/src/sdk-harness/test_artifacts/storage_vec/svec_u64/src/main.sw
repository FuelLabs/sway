contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_u64_push(value: u64);
    #[storage(read)]
    fn vec_u64_get(index: u64) -> Option<u64>;
    #[storage(read, write)]
    fn vec_u64_pop() -> Option<u64>;
    #[storage(read, write)]
    fn vec_u64_remove(index: u64) -> Result<u64, StorageVecError>;
    #[storage(read, write)]
    fn vec_u64_swap_remove(index: u64) -> Result<u64, StorageVecError>;
    #[storage(read, write)]
    fn vec_u64_insert(index: u64, value: u64) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_u64_len() -> u64;
    #[storage(read)]
    fn vec_u64_is_empty() -> bool;
    #[storage(write)]
    fn vec_u64_clear();
}

storage {
    vec_u64: StorageVec<u64>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_u64_push(value: u64) {
        storage.vec_u64.push(value);
    }
    #[storage(read)]
    fn vec_u64_get(index: u64) -> Option<u64> {
        storage.vec_u64.get(index)
    }
    #[storage(read, write)]
    fn vec_u64_pop() -> Option<u64> {
        storage.vec_u64.pop()
    }
    #[storage(read, write)]
    fn vec_u64_remove(index: u64) -> Result<u64, StorageVecError> {
        let res: Result<u64, StorageVecError> = storage.vec_u64.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_u64_swap_remove(index: u64) -> Result<u64, StorageVecError> {
        storage.vec_u64.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_u64_insert(index: u64, value: u64) -> Result<(), StorageVecError> {
        storage.vec_u64.insert(index, value)
    }
    #[storage(read)]
    fn vec_u64_len() -> u64 {
        storage.vec_u64.len()
    }
    #[storage(read)]
    fn vec_u64_is_empty() -> bool {
        storage.vec_u64.is_empty()
    }
    #[storage(write)]
    fn vec_u64_clear() {
        storage.vec_u64.clear();
    }
}