contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_u16_push(value: u16);
    #[storage(read)]
    fn vec_u16_get(index: u64) -> Option<u16>;
    #[storage(read, write)]
    fn vec_u16_pop() -> Option<u16>;
    #[storage(read, write)]
    fn vec_u16_remove(index: u64) -> Result<u16, StorageVecError>;
    #[storage(read, write)]
    fn vec_u16_swap_remove(index: u64) -> Result<u16, StorageVecError>;
    #[storage(read, write)]
    fn vec_u16_insert(index: u64, value: u16) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_u16_len() -> u64;
    #[storage(read)]
    fn vec_u16_is_empty() -> bool;
    #[storage(write)]
    fn vec_u16_clear();
}

storage {
    vec_u16: StorageVec<u16>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_u16_push(value: u16) {
        storage.vec_u16.push(value);
    }
    #[storage(read)]
    fn vec_u16_get(index: u64) -> Option<u16> {
        storage.vec_u16.get(index)
    }
    #[storage(read, write)]
    fn vec_u16_pop() -> Option<u16> {
        storage.vec_u16.pop()
    }
    #[storage(read, write)]
    fn vec_u16_remove(index: u64) -> Result<u16, StorageVecError> {
        let res: Result<u16, StorageVecError> = storage.vec_u16.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_u16_swap_remove(index: u64) -> Result<u16, StorageVecError> {
        storage.vec_u16.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_u16_insert(index: u64, value: u16) -> Result<(), StorageVecError> {
        storage.vec_u16.insert(index, value)
    }
    #[storage(read)]
    fn vec_u16_len() -> u64 {
        storage.vec_u16.len()
    }
    #[storage(read)]
    fn vec_u16_is_empty() -> bool {
        storage.vec_u16.is_empty()
    }
    #[storage(write)]
    fn vec_u16_clear() {
        storage.vec_u16.clear();
    }
}