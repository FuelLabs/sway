contract;

use std::{option::Option, result::Result, storage::StorageVec};

abi MyContract {
    #[storage(read, write)]
    fn tuple_push(value: (u8, u8, u8));
    #[storage(read)]
    fn tuple_get(index: u64) -> (u8, u8, u8);
    #[storage(read, write)]
    fn tuple_pop() -> (u8, u8, u8);
    #[storage(read, write)]
    fn tuple_remove(index: u64) -> (u8, u8, u8);
    #[storage(read, write)]
    fn tuple_swap_remove(index: u64) -> (u8, u8, u8);
    #[storage(read, write)]
    fn tuple_set(index: u64, value: (u8, u8, u8));
    #[storage(read, write)]
    fn tuple_insert(index: u64, value: (u8, u8, u8));
    #[storage(read)]
    fn tuple_len() -> u64;
    #[storage(read)]
    fn tuple_is_empty() -> bool;
    #[storage(write)]
    fn tuple_clear();
}

storage {
    my_vec: StorageVec<(u8, u8, u8)> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn tuple_push(value: (u8, u8, u8)) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn tuple_get(index: u64) -> (u8, u8, u8) {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn tuple_pop() -> (u8, u8, u8) {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn tuple_remove(index: u64) -> (u8, u8, u8) {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn tuple_swap_remove(index: u64) -> (u8, u8, u8) {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn tuple_set(index: u64, value: (u8, u8, u8)) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn tuple_insert(index: u64, value: (u8, u8, u8)) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn tuple_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn tuple_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn tuple_clear() {
        storage.my_vec.clear();
    }
}
