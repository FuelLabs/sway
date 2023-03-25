contract;

use core::experimental::storage::*;
use std::experimental::storage::*;

abi MyContract {
    #[storage(read, write)]
    fn push(value: u8);

    #[storage(read, write)]
    fn pop() -> u8;

    #[storage(read)]
    fn get(index: u64) -> u8;

    #[storage(read, write)]
    fn remove(index: u64) -> u8;

    #[storage(read, write)]
    fn swap_remove(index: u64) -> u8;

    #[storage(read, write)]
    fn set(index: u64, value: u8);

    #[storage(read, write)]
    fn insert(index: u64, value: u8);

    #[storage(read)]
    fn len() -> u64;

    #[storage(read)]
    fn is_empty() -> bool;

    #[storage(write)]
    fn clear();
}

storage {
    my_vec: StorageVec<u8> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn push(value: u8) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn pop() -> u8 {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read)]
    fn get(index: u64) -> u8 {
        storage.my_vec.get(index).unwrap().read()
    }

    #[storage(read, write)]
    fn remove(index: u64) -> u8 {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn swap_remove(index: u64) -> u8 {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn set(index: u64, value: u8) {
        storage.my_vec.set(index, value);
    }

    #[storage(read, write)]
    fn insert(index: u64, value: u8) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read)]
    fn len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(write)]
    fn clear() {
        storage.my_vec.clear();
    }
}
