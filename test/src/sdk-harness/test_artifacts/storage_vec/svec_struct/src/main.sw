contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

struct TestStruct {
    A: bool,
    B: u64,
}

abi MyContract {
    #[storage(read, write)]
    fn vec_struct_push(value: TestStruct);
    #[storage(read)]
    fn vec_struct_get(index: u64) -> Option<TestStruct>;
    #[storage(read, write)]
    fn vec_struct_pop() -> Option<TestStruct>;
    #[storage(read, write)]
    fn vec_struct_remove(index: u64) -> Result<TestStruct, StorageVecError>;
    #[storage(read, write)]
    fn vec_struct_swap_remove(index: u64) -> Result<TestStruct, StorageVecError>;
    #[storage(read, write)]
    fn vec_struct_insert(index: u64, value: TestStruct) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_struct_len() -> u64;
    #[storage(read)]
    fn vec_struct_is_empty() -> bool;
    #[storage(write)]
    fn vec_struct_clear();
}

storage {
    vec_struct: StorageVec<TestStruct>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_struct_push(value: TestStruct) {
        storage.vec_struct.push(value);
    }
    #[storage(read)]
    fn vec_struct_get(index: u64) -> Option<TestStruct> {
        storage.vec_struct.get(index)
    }
    #[storage(read, write)]
    fn vec_struct_pop() -> Option<TestStruct> {
        storage.vec_struct.pop()
    }
    #[storage(read, write)]
    fn vec_struct_remove(index: u64) -> Result<TestStruct, StorageVecError> {
        let res: Result<TestStruct, StorageVecError> = storage.vec_struct.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_struct_swap_remove(index: u64) -> Result<TestStruct, StorageVecError> {
        storage.vec_struct.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_struct_insert(index: u64, value: TestStruct) -> Result<(), StorageVecError> {
        storage.vec_struct.insert(index, value)
    }
    #[storage(read)]
    fn vec_struct_len() -> u64 {
        storage.vec_struct.len()
    }
    #[storage(read)]
    fn vec_struct_is_empty() -> bool {
        storage.vec_struct.is_empty()
    }
    #[storage(write)]
    fn vec_struct_clear() {
        storage.vec_struct.clear();
    }
}