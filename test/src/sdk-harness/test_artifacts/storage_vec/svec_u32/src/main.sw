contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_u32_push(value: u32);
    #[storage(read)]
    fn vec_u32_get(index: u64) -> Option<u32>;
    #[storage(read, write)]
    fn vec_u32_pop() -> Option<u32>;
    #[storage(read, write)]
    fn vec_u32_remove(index: u64) -> Result<u32, StorageVecError>;
    #[storage(read, write)]
    fn vec_u32_swap_remove(index: u64) -> Result<u32, StorageVecError>;
    #[storage(read, write)]
    fn vec_u32_insert(index: u64, value: u32) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_u32_len() -> u64;
    #[storage(read)]
    fn vec_u32_is_empty() -> bool;
    #[storage(write)]
    fn vec_u32_clear();
}

storage {
    vec_u32: StorageVec<u32>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_u32_push(value: u32) {
        storage.vec_u32.push(value);
    }
    #[storage(read)]
    fn vec_u32_get(index: u64) -> Option<u32> {
        storage.vec_u32.get(index)
    }
    #[storage(read, write)]
    fn vec_u32_pop() -> Option<u32> {
        storage.vec_u32.pop()
    }
    #[storage(read, write)]
    fn vec_u32_remove(index: u64) -> Result<u32, StorageVecError> {
        let res: Result<u32, StorageVecError> = storage.vec_u32.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_u32_swap_remove(index: u64) -> Result<u32, StorageVecError> {
        storage.vec_u32.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_u32_insert(index: u64, value: u32) -> Result<(), StorageVecError> {
        storage.vec_u32.insert(index, value)
    }
    #[storage(read)]
    fn vec_u32_len() -> u64 {
        storage.vec_u32.len()
    }
    #[storage(read)]
    fn vec_u32_is_empty() -> bool {
        storage.vec_u32.is_empty()
    }
    #[storage(write)]
    fn vec_u32_clear() {
        storage.vec_u32.clear();
    }
}