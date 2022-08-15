contract;

use std::{option::Option, result::Result, storage::StorageVec};

struct TestStruct {
    A: bool,
    B: u16,
}

abi MyContract {
    #[storage(read, write)]
    fn struct_push(value: TestStruct);
    #[storage(read)]
    fn struct_get(index: u64) -> TestStruct;
    #[storage(read, write)]
    fn struct_pop() -> TestStruct;
    #[storage(read, write)]
    fn struct_remove(index: u64) -> TestStruct;
    #[storage(read, write)]
    fn struct_swap_remove(index: u64) -> TestStruct;
    #[storage(read, write)]
    fn struct_set(index: u64, value: TestStruct);
    #[storage(read, write)]
    fn struct_insert(index: u64, value: TestStruct);
    #[storage(read)]
    fn struct_len() -> u64;
    #[storage(read)]
    fn struct_is_empty() -> bool;
    #[storage(write)]
    fn struct_clear();
}

storage {
    my_vec: StorageVec<TestStruct> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn struct_push(value: TestStruct) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn struct_get(index: u64) -> TestStruct {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn struct_pop() -> TestStruct {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn struct_remove(index: u64) -> TestStruct {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn struct_swap_remove(index: u64) -> TestStruct {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn struct_set(index: u64, value: TestStruct) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn struct_insert(index: u64, value: TestStruct) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn struct_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn struct_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn struct_clear() {
        storage.my_vec.clear();
    }
}
