contract;

use std::{option::Option, result::Result, storage::StorageVec};

abi MyContract {
    #[storage(read, write)]
    fn bool_push(value: bool);
    #[storage(read)]
    fn bool_get(index: u64) -> bool;
    #[storage(read, write)]
    fn bool_pop() -> bool;
    #[storage(read, write)]
    fn bool_remove(index: u64) -> bool;
    #[storage(read, write)]
    fn bool_swap_remove(index: u64) -> bool;
    #[storage(read, write)]
    fn bool_set(index: u64, value: bool);
    #[storage(read, write)]
    fn bool_insert(index: u64, value: bool);
    #[storage(read)]
    fn bool_len() -> u64;
    #[storage(read)]
    fn bool_is_empty() -> bool;
    #[storage(write)]
    fn bool_clear();
}

storage {
    my_vec: StorageVec<bool> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn bool_push(value: bool) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn bool_get(index: u64) -> bool {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn bool_pop() -> bool {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn bool_remove(index: u64) -> bool {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn bool_swap_remove(index: u64) -> bool {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn bool_set(index: u64, value: bool) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn bool_insert(index: u64, value: bool) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn bool_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn bool_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn bool_clear() {
        storage.my_vec.clear();
    }
}
