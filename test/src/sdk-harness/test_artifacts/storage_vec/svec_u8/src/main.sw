contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn u8_push(value: u8);
    #[storage(read)]
    fn u8_get(index: u64) -> u8;
    #[storage(read, write)]
    fn u8_pop() -> u8;
    #[storage(read, write)]
    fn u8_remove(index: u64) -> u8;
    #[storage(read, write)]
    fn u8_swap_remove(index: u64) -> u8;
    #[storage(read, write)]
    fn u8_set(index: u64, value: u8);
    #[storage(read, write)]
    fn u8_insert(index: u64, value: u8);
    #[storage(read)]
    fn u8_len() -> u64;
    #[storage(read)]
    fn u8_is_empty() -> bool;
    #[storage(write)]
    fn u8_clear();
}

storage {
    my_vec: StorageVec<u8> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn u8_push(value: u8) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn u8_get(index: u64) -> u8 {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn u8_pop() -> u8 {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn u8_remove(index: u64) -> u8 {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn u8_swap_remove(index: u64) -> u8 {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn u8_set(index: u64, value: u8) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn u8_insert(index: u64, value: u8) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn u8_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn u8_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn u8_clear() {
        storage.my_vec.clear();
    }
}
