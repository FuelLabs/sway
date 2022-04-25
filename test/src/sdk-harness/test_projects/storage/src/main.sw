contract;

use std::{
    storage::{get, store}, hash::*,
};

////////////////////////////////////////
// Helper functions
////////////////////////////////////////

/// Compute the storage slot for an address's deposits.
fn get_storage_key<T>(value: T, delimiter: b256) -> b256 {
    hash_pair(delimiter, value, HashMethod::Sha256)
}

pub enum StorageEnum {
    One: u64,
    Two: bool,
    Three: b256,
}

pub struct StorageStruct {
  useless_number: u64,
  status: bool,
}

/// Storage delimiters
const S_1: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const S_2: b256 = 0x0000000000000000000000000000000000000000000000000000000000000002;
const S_3: b256 = 0x0000000000000000000000000000000000000000000000000000000000000003;
const S_4: b256 = 0x0000000000000000000000000000000000000000000000000000000000000004;
const S_5: b256 = 0x0000000000000000000000000000000000000000000000000000000000000005;

storage {
    val_1: u64,
    val_2: b256,
    val_3: StorageEnum,
    val_4: StorageStruct,
    val_5: (bool, u64),
    val_6: [b256; 7],
}

abi StorageTest {
    fn store_u64(value: u64);
    fn get_u64() -> u64;
    fn store_b256(value: b256);
    fn get_b256() -> b256;
    // fn store_enum(value: StorageEnum);
    // fn get_enum() -> StorageEnum;
    // fn store_struct(value: StorageStruct);
    // fn get_struct() -> StorageStruct;
    // fn store_tuple(value: b256);
    // fn get_tuple() -> (bool, u64);
    // fn store_array(value: b256);
    // fn get_array() -> [b256; 7];
}

impl StorageTest for Contract {
    // primitive types can use the new storage syntax
    fn store_u64(value: u64) {
        storage.val_1 = value;
    }

    fn get_u64() -> u64 {
        storage.val_1
    }

    fn store_b256(value: b256) {
        store(S_1, value);
    }

    fn get_b256() -> b256 {
        get::<b256>(S_1)
    }

    // fn store_enum(value: StorageEnum) {
    //     store(STORAGE_KEY, value);
    // }

    // fn get_enum() -> StorageEnum {
    //     get::<StorageEnum>(STORAGE_KEY)
    // }

    // fn store_struct(value: StorageStruct) {
    //     store(STORAGE_KEY, value);
    // }

    // fn get_struct() -> StorageStruct {
    //     get::<StorageStruct>(STORAGE_KEY)
    // }

    // fn store_tuple(value: b256) {
    //     store(STORAGE_KEY, value);
    // }

    // fn get_tuple() -> (bool, u64) {
    //     get::<(bool, u64)>(STORAGE_KEY)
    // }

    // fn store_array(value: b256) {
    //     store(STORAGE_KEY, value);
    // }

    // fn get_array() -> [b256; 7] {
    //     get::<[b256; 7]>(STORAGE_KEY)
    // }
}
