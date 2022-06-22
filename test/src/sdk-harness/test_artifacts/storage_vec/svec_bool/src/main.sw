contract;

use std::option::*;
use std::result::*;use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_bool_push(value: bool);
    #[storage(read)]
    fn vec_bool_get(index: u64) -> Option<bool>;
    #[storage(read, write)]
    fn vec_bool_pop() -> Option<bool>;
    #[storage(read, write)]
    fn vec_bool_remove(index: u64) -> Result<bool, StorageVecError>;
    #[storage(read, write)]
    fn vec_bool_swap_remove(index: u64) -> Result<bool, StorageVecError>;
    #[storage(read, write)]
    fn vec_bool_insert(index: u64, value: bool) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_bool_len() -> u64;
    #[storage(read)]
    fn vec_bool_is_empty() -> bool;
    #[storage(write)]
    fn vec_bool_clear();
}

storage {
    vec_bool: StorageVec<bool>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_bool_push(value: bool) {
        storage.vec_bool.push(value);
    }
    #[storage(read)]
    fn vec_bool_get(index: u64) -> Option<bool> {
        storage.vec_bool.get(index)
    }
    #[storage(read, write)]
    fn vec_bool_pop() -> Option<bool> {
        storage.vec_bool.pop()
    }
    #[storage(read, write)]
    fn vec_bool_remove(index: u64) -> Result<bool, StorageVecError> {
        let res: Result<bool, StorageVecError> = storage.vec_bool.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_bool_swap_remove(index: u64) -> Result<bool, StorageVecError> {
        storage.vec_bool.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_bool_insert(index: u64, value: bool) -> Result<(), StorageVecError> {
        storage.vec_bool.insert(index, value)
    }
    #[storage(read)]
    fn vec_bool_len() -> u64 {
        storage.vec_bool.len()
    }
    #[storage(read)]
    fn vec_bool_is_empty() -> bool {
        storage.vec_bool.is_empty()
    }
    #[storage(write)]
    fn vec_bool_clear() {
        storage.vec_bool.clear();
    }
}