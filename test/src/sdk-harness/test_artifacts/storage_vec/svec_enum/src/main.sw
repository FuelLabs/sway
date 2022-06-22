contract;

use std::option::*;
use std::result::*;
use std::storage::{StorageVec, StorageVecError};

enum TestEnum {
    A: (),
    B: (),
}

abi MyContract {
    #[storage(read, write)]
    fn vec_enum_push(value: TestEnum);
    #[storage(read)]
    fn vec_enum_get(index: u64) -> Option<TestEnum>;
    #[storage(read, write)]
    fn vec_enum_pop() -> Option<TestEnum>;
    #[storage(read, write)]
    fn vec_enum_remove(index: u64) -> Result<TestEnum, StorageVecError>;
    #[storage(read, write)]
    fn vec_enum_swap_remove(index: u64) -> Result<TestEnum, StorageVecError>;
    #[storage(read, write)]
    fn vec_enum_insert(index: u64, value: TestEnum) -> Result<(), StorageVecError>;
    #[storage(read)]
    fn vec_enum_len() -> u64;
    #[storage(read)]
    fn vec_enum_is_empty() -> bool;
    #[storage(write)]
    fn vec_enum_clear();
}

storage {
    vec_enum: StorageVec<TestEnum>,
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn vec_enum_push(value: TestEnum) {
        storage.vec_enum.push(value);
    }
    #[storage(read)]
    fn vec_enum_get(index: u64) -> Option<TestEnum> {
        storage.vec_enum.get(index)
    }
    #[storage(read, write)]
    fn vec_enum_pop() -> Option<TestEnum> {
        storage.vec_enum.pop()
    }
    #[storage(read, write)]
    fn vec_enum_remove(index: u64) -> Result<TestEnum, StorageVecError> {
        let res: Result<TestEnum, StorageVecError> = storage.vec_enum.remove(index);
        res
    }
    #[storage(read, write)]
    fn vec_enum_swap_remove(index: u64) -> Result<TestEnum, StorageVecError> {
        storage.vec_enum.swap_remove(index)
    }
    #[storage(read, write)]
    fn vec_enum_insert(index: u64, value: TestEnum) -> Result<(), StorageVecError> {
        storage.vec_enum.insert(index, value)
    }
    #[storage(read)]
    fn vec_enum_len() -> u64 {
        storage.vec_enum.len()
    }
    #[storage(read)]
    fn vec_enum_is_empty() -> bool {
        storage.vec_enum.is_empty()
    }
    #[storage(write)]
    fn vec_enum_clear() {
        storage.vec_enum.clear();
    }
}