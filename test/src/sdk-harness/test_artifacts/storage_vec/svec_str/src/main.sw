contract;

use std::{option::Option, result::Result, storage::StorageVec};

abi MyContract {
    #[storage(read, write)]
    fn str_push(value: str[4]);
    #[storage(read)]
    fn str_get(index: u64) -> str[4];
    #[storage(read, write)]
    fn str_pop() -> str[4];
    #[storage(read, write)]
    fn str_remove(index: u64) -> str[4];
    #[storage(read, write)]
    fn str_swap_remove(index: u64) -> str[4];
    #[storage(read, write)]
    fn str_set(index: u64, value: str[4]);
    #[storage(read, write)]
    fn str_insert(index: u64, value: str[4]);
    #[storage(read)]
    fn str_len() -> u64;
    #[storage(read)]
    fn str_is_empty() -> bool;
    #[storage(write)]
    fn str_clear();
}

storage {
    my_vec: StorageVec<str[4]> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn str_push(value: str[4]) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn str_get(index: u64) -> str[4] {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn str_pop() -> str[4] {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn str_remove(index: u64) -> str[4] {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn str_swap_remove(index: u64) -> str[4] {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn str_set(index: u64, value: str[4]) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn str_insert(index: u64, value: str[4]) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn str_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn str_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn str_clear() {
        storage.my_vec.clear();
    }
}
