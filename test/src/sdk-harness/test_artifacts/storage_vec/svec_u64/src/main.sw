contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn u64_push(value: u64);
    #[storage(read)]
    fn u64_get(index: u64) -> u64;
    #[storage(read, write)]
    fn u64_pop() -> u64;
    #[storage(read, write)]
    fn u64_remove(index: u64) -> u64;
    #[storage(read, write)]
    fn u64_swap_remove(index: u64) -> u64;
    #[storage(read, write)]
    fn u64_set(index: u64, value: u64);
    #[storage(read, write)]
    fn u64_insert(index: u64, value: u64);
    #[storage(read)]
    fn u64_len() -> u64;
    #[storage(read)]
    fn u64_is_empty() -> bool;
    #[storage(write)]
    fn u64_clear();
}

storage {
    my_vec: StorageVec<u64> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn u64_push(value: u64) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn u64_get(index: u64) -> u64 {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn u64_pop() -> u64 {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn u64_remove(index: u64) -> u64 {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn u64_swap_remove(index: u64) -> u64 {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn u64_set(index: u64, value: u64) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn u64_insert(index: u64, value: u64) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn u64_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn u64_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn u64_clear() {
        storage.my_vec.clear();
    }
}
