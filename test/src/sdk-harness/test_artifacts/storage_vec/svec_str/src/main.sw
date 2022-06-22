contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_str_push(value: str[4]);
    #[storage(read)]
    fn vec_str_get(index: u64) -> Option<str[4]>;
    #[storage(read, write)]
    fn vec_str_pop() -> Option<str[4]>;
    #[storage(read, write)]
    fn vec_str_remove(index: u64) -> Result<str[4], StorageVecError>;
    #[storage(read, write)]
    fn vec_str_swap_remove(index: u64) -> Result<str[4], StorageVecError>;
    #[storage(read, write)]
    fn vec_str_insert(index: u64, value: str[4]) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_str_len() -> u64;
    #[storage(read)]
    fn vec_str_is_empty() -> bool;
    #[storage(write)]
    fn vec_str_clear();
}

storage {
    vec_str: StorageVec<str[4]>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_str_push(value: str[4]) {
        storage.vec_str.push(value);
    }
    #[storage(read)]
    fn vec_str_get(index: u64) -> Option<str[4]> {
        storage.vec_str.get(index)
    }
    #[storage(read, write)]
    fn vec_str_pop() -> Option<str[4]> {
        storage.vec_str.pop()
    }
    #[storage(read, write)]
    fn vec_str_remove(index: u64) -> Result<str[4], StorageVecError> {
        let res: Result<str[4], StorageVecError> = storage.vec_str.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_str_swap_remove(index: u64) -> Result<str[4], StorageVecError> {
        storage.vec_str.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_str_insert(index: u64, value: str[4]) -> Result<(), StorageVecError> {
        storage.vec_str.insert(index, value)
    }
    #[storage(read)]
    fn vec_str_len() -> u64 {
        storage.vec_str.len()
    }
    #[storage(read)]
    fn vec_str_is_empty() -> bool {
        storage.vec_str.is_empty()
    }
    #[storage(write)]
    fn vec_str_clear() {
        storage.vec_str.clear();
    }
}