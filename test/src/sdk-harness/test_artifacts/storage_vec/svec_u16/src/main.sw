contract;

use std::{option::Option, result::Result, storage::StorageVec};

abi MyContract {
    #[storage(read, write)]
    fn u16_push(value: u16);
    #[storage(read)]
    fn u16_get(index: u64) -> u16;
    #[storage(read, write)]
    fn u16_pop() -> u16;
    #[storage(read, write)]
    fn u16_remove(index: u64) -> u16;
    #[storage(read, write)]
    fn u16_swap_remove(index: u64) -> u16;
    #[storage(read, write)]
    fn u16_set(index: u64, value: u16);
    #[storage(read, write)]
    fn u16_insert(index: u64, value: u16);
    #[storage(read)]
    fn u16_len() -> u64;
    #[storage(read)]
    fn u16_is_empty() -> bool;
    #[storage(write)]
    fn u16_clear();
}

storage {
    my_vec: StorageVec<u16> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn u16_push(value: u16) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn u16_get(index: u64) -> u16 {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn u16_pop() -> u16 {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn u16_remove(index: u64) -> u16 {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn u16_swap_remove(index: u64) -> u16 {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn u16_set(index: u64, value: u16) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn u16_insert(index: u64, value: u16) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn u16_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn u16_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn u16_clear() {
        storage.my_vec.clear();
    }
}
