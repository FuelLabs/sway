contract;

use std::Result::*;
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
    fn test_function() -> bool;
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
    #[store(write)]
    fn vec_u8_push(value: u8) {
        storage.vec_u8.push(value);
    }
    #[store(read)]
    fn vec_u8_get(index: u64) {
        storage.vec_u8.get(index);
    }
    #[store(write)]
    fn vec_u8_pop() {
        storage.vec_u8.pop();
    }
    #[store(read, write)]
    fn vec_u8_remove(index: u64) -> Result<V, StorageVecError> {
        storage.vec_u8.remove(index)
    }
    #[store(read, write)]
    fn vec_u8_swap_remove(index: u64) -> Result<V, StorageVecError> {
        storage.vec_u8.swap_remove(index)
    }
}
