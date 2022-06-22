contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_array_push(value: [u64; 2]);
    #[storage(read)]
    fn vec_array_get(index: u64) -> Option<[u64; 2]>;
    #[storage(read, write)]
    fn vec_array_pop() -> Option<[u64; 2]>;
    #[storage(read, write)]
    fn vec_array_remove(index: u64) -> Result<[u64; 2], StorageVecError>;
    #[storage(read, write)]
    fn vec_array_swap_remove(index: u64) -> Result<[u64; 2], StorageVecError>;
    #[storage(read, write)]
    fn vec_array_insert(index: u64, value: [u64; 2]) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_array_len() -> u64;
    #[storage(read)]
    fn vec_array_is_empty() -> bool;
    #[storage(write)]
    fn vec_array_clear();
}

storage {
    vec_array: StorageVec<[u64; 2]>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_array_push(value: [u64; 2]) {
        storage.vec_array.push(value);
    }
    #[storage(read)]
    fn vec_array_get(index: u64) -> Option<[u64; 2]> {
        storage.vec_array.get(index)
    }
    #[storage(read, write)]
    fn vec_array_pop() -> Option<[u64; 2]> {
        storage.vec_array.pop()
    }
    #[storage(read, write)]
    fn vec_array_remove(index: u64) -> Result<[u64; 2], StorageVecError> {
        let res: Result<[u64; 2], StorageVecError> = storage.vec_array.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_array_swap_remove(index: u64) -> Result<[u64; 2], StorageVecError> {
        storage.vec_array.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_array_insert(index: u64, value: [u64; 2]) -> Result<(), StorageVecError> {
        storage.vec_array.insert(index, value)
    }
    #[storage(read)]
    fn vec_array_len() -> u64 {
        storage.vec_array.len()
    }
    #[storage(read)]
    fn vec_array_is_empty() -> bool {
        storage.vec_array.is_empty()
    }
    #[storage(write)]
    fn vec_array_clear() {
        storage.vec_array.clear();
    }
}