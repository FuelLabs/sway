contract;

use std::result::*;
use std::storage::{StorageVec, StorageVecError};

enum TestEnum {
    A: (),
    B: (),
}

struct TestStruct {
    A: bool,
    B: u64,
}

abi MyContract {
    #[storage(write)]
    fn vec_u8_push(value: u8);
    #[storage(read)]
    fn vec_u8_get(index: u64);
    #[storage(write)]
    fn vec_u8_pop();
    #[storage(read, write)]
    fn vec_u8_remove(index: u64) -> Result<u8, StorageVecError>;
    #[storage(read, write)]
    fn vec_u8_swap_remove(index: u64) -> Result<u8, StorageVecError>;
    #[storage(read, write)]
    fn vec_u8_insert(index: u64, value: u8) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_u8_len() -> u64;
    #[storage(read)]
    fn vec_u8_is_empty() -> bool;
    #[storage(write)]
    fn vec_u8_clear();

    
}

storage {
    vec_u8: StorageVec<u8>,
    vec_u16: StorageVec<u16>,
    vec_u32: StorageVec<u32>,
    vec_u64: StorageVec<u64>,
    vec_bool: StorageVec<bool>,
    vec_str4: StorageVec<str[4]>,
    vec_b256: StorageVec<b256>,
    vec_u64_tuple: StorageVec<(u64, u64)>,
    vec_u64_array: StorageVec<[u64; 2]>,
    vec_enum: StorageVec<TestEnum>,
    vec_struct: StorageVec<TestStruct>,
}

impl MyContract for Contract {
    #[storage(write)]
    fn vec_u8_push(value: u8) {
        storage.vec_u8.push(value);
    }
    #[storage(read)]
    fn vec_u8_get(index: u64) {
        storage.vec_u8.get(index);
    }
    #[storage(write)]
    fn vec_u8_pop() {
        storage.vec_u8.pop();
    }
    #[storage(read, write)]
    fn vec_u8_remove(index: u64) -> Result<u8, StorageVecError> {
        storage.vec_u8.remove(index)
    }
    #[storage(read, write)]
    fn vec_u8_swap_remove(index: u64) -> Result<u8, StorageVecError> {
        storage.vec_u8.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_u8_insert(index: u64, value: u8) -> Result<(), StorageVecError> {
        storage.vec_u8.insert(index, value)
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
