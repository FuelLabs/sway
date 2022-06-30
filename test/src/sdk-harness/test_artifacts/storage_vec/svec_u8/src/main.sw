contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

abi MyContract {
    #[storage(read, write)]
    fn vec_u8_push(value: u8);
    #[storage(read)]
    fn vec_u8_get(index: u64) -> u8;
    #[storage(read, write)]
    fn vec_u8_pop() -> u8;
    #[storage(read, write)]
    fn vec_u8_remove(index: u64) -> u8;
    #[storage(read, write)]
    fn vec_u8_swap_remove(index: u64) -> u8;
    #[storage(read, write)]
    fn vec_u8_insert(index: u64, value: u8);
    #[storage(read)]
    fn vec_u8_len() -> u64;
    #[storage(read)]
    fn vec_u8_is_empty() -> bool;
    #[storage(write)]
    fn vec_u8_clear();
}

storage {
    vec_u8: StorageVec<u8>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_u8_push(value: u8) {
        storage.vec_u8.push(value);
    }
    #[storage(read)]
    fn vec_u8_get(index: u64) -> u8 {
        let item: Option<u8> = storage.vec_u8.get(index);
        item.unwrap()
    }
    #[storage(read, write)]
    fn vec_u8_pop() -> u8 {
        let res: Option<u8> = storage.vec_u8.pop();
        res.unwrap()
    }
    #[storage(read, write)]
    fn vec_u8_remove(index: u64) -> u8 {
        let res: Result<u8, StorageVecError> = storage.vec_u8.remove(index);
        res.unwrap()
    }
    #[storage(read, write)]
    fn vec_u8_swap_remove(index: u64) -> u8 {
        let res: Result<u8, StorageVecError> = storage.vec_u8.swap_remove(index);
        res.unwrap()
    }
    #[storage(read, write)]
    fn vec_u8_insert(index: u64, value: u8) {
        let _ = storage.vec_u8.insert(index, value);
    }
    #[storage(read)]
    fn vec_u8_len() -> u64 {
        storage.vec_u8.len()
    }
    #[storage(read)]
    fn vec_u8_is_empty() -> bool {
        storage.vec_u8.is_empty()
    }
    #[storage(write)]
    fn vec_u8_clear() {
        storage.vec_u8.clear();
    }
}
