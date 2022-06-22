contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_tuple_push(value: (u64, u64));
    #[storage(read)]
    fn vec_tuple_get(index: u64) -> Option<(u64, u64)>;
    #[storage(read, write)]
    fn vec_tuple_pop() -> Option<(u64, u64)>;
    #[storage(read, write)]
    fn vec_tuple_remove(index: u64) -> Result<(u64, u64), StorageVecError>;
    #[storage(read, write)]
    fn vec_tuple_swap_remove(index: u64) -> Result<(u64, u64), StorageVecError>;
    #[storage(read, write)]
    fn vec_tuple_insert(index: u64, value: (u64, u64)) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_tuple_len() -> u64;
    #[storage(read)]
    fn vec_tuple_is_empty() -> bool;
    #[storage(write)]
    fn vec_tuple_clear();
}

storage {
    vec_tuple: StorageVec<(u64, u64)>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_tuple_push(value: (u64, u64)) {
        storage.vec_tuple.push(value);
    }
    #[storage(read)]
    fn vec_tuple_get(index: u64) -> Option<(u64, u64)> {
        storage.vec_tuple.get(index)
    }
    #[storage(read, write)]
    fn vec_tuple_pop() -> Option<(u64, u64)> {
        storage.vec_tuple.pop()
    }
    #[storage(read, write)]
    fn vec_tuple_remove(index: u64) -> Result<(u64, u64), StorageVecError> {
        let res: Result<(u64, u64), StorageVecError> = storage.vec_tuple.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_tuple_swap_remove(index: u64) -> Result<(u64, u64), StorageVecError> {
        storage.vec_tuple.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_tuple_insert(index: u64, value: (u64, u64)) -> Result<(), StorageVecError> {
        storage.vec_tuple.insert(index, value)
    }
    #[storage(read)]
    fn vec_tuple_len() -> u64 {
        storage.vec_tuple.len()
    }
    #[storage(read)]
    fn vec_tuple_is_empty() -> bool {
        storage.vec_tuple.is_empty()
    }
    #[storage(write)]fn vec_tuple_clear() {
        storage.vec_tuple.clear();
    }
}