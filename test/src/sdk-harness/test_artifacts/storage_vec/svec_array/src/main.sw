contract;

use std::{option::Option, result::Result, storage::StorageVec};

abi MyContract {
    #[storage(read, write)]
    fn array_push(value: [u8; 3]);
    #[storage(read)]
    fn array_get(index: u64) -> [u8; 3];
    #[storage(read, write)]
    fn array_pop() -> [u8; 3];
    #[storage(read, write)]
    fn array_remove(index: u64) -> [u8; 3];
    #[storage(read, write)]
    fn array_swap_remove(index: u64) -> [u8; 3];
    #[storage(read, write)]
    fn array_set(index: u64, value: [u8; 3]);
    #[storage(read, write)]
    fn array_insert(index: u64, value: [u8; 3]);
    #[storage(read)]
    fn array_len() -> u64;
    #[storage(read)]
    fn array_is_empty() -> bool;
    #[storage(write)]
    fn array_clear();
}

storage {
    my_vec: StorageVec<[u8; 3]> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn array_push(value: [u8; 3]) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn array_get(index: u64) -> [u8; 3] {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn array_pop() -> [u8; 3] {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn array_remove(index: u64) -> [u8; 3] {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn array_swap_remove(index: u64) -> [u8; 3] {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn array_set(index: u64, value: [u8; 3]) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn array_insert(index: u64, value: [u8; 3]) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn array_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn array_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn array_clear() {
        storage.my_vec.clear();
    }
}
