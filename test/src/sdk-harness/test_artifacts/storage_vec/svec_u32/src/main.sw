contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn u32_push(value: u32);
    #[storage(read)]
    fn u32_get(index: u64) -> u32;
    #[storage(read, write)]
    fn u32_pop() -> u32;
    #[storage(read, write)]
    fn u32_remove(index: u64) -> u32;
    #[storage(read, write)]
    fn u32_swap_remove(index: u64) -> u32;
    #[storage(read, write)]
    fn u32_set(index: u64, value: u32);
    #[storage(read, write)]
    fn u32_insert(index: u64, value: u32);
    #[storage(read)]
    fn u32_len() -> u64;
    #[storage(read)]
    fn u32_is_empty() -> bool;
    #[storage(write)]
    fn u32_clear();
}

storage {
    my_vec: StorageVec<u32> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn u32_push(value: u32) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn u32_get(index: u64) -> u32 {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn u32_pop() -> u32 {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn u32_remove(index: u64) -> u32 {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn u32_swap_remove(index: u64) -> u32 {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn u32_set(index: u64, value: u32) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn u32_insert(index: u64, value: u32) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn u32_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn u32_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn u32_clear() {
        storage.my_vec.clear();
    }
}
