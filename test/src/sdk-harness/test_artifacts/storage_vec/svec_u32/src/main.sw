contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn u32_push(value: u32);

    #[storage(read, write)]
    fn u32_push_other(value: u32);

    #[storage(write)]
    fn u32_clear();
    #[storage(read)]
    fn u32_get(index: u64) -> u32;

    #[storage(read)]
    fn u32_len() -> u64;

    #[storage(read)]
    fn u32_is_empty() -> bool;

    #[storage(read, write)]
    fn u32_remove(index: u64) -> u32;

    #[storage(read, write)]
    fn u32_insert(index: u64, value: u32);

    #[storage(read, write)]
    fn u32_pop() -> u32;

    #[storage(read, write)]
    fn u32_swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn u32_swap_remove(index: u64) -> u32;

    #[storage(read, write)]
    fn u32_set(index: u64, value: u32);

    #[storage(read)]
    fn u32_first() -> u32;

    #[storage(read)]
    fn u32_last() -> u32;

    #[storage(read, write)]
    fn u32_reverse();

    #[storage(read, write)]
    fn u32_fill(value: u32);

    #[storage(read, write)]
    fn u32_resize(new_len: u64, value: u32);

    #[storage(read, write)]
    fn u32_append();
}

storage {
    my_vec: StorageVec<u32> = StorageVec {},
    my_other_vec: StorageVec<u32> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn u32_push(value: u32) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn u32_push_other(value: u32) {
        storage.my_other_vec.push(value);
    }

    #[storage(write)]
    fn u32_clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn u32_get(index: u64) -> u32 {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn u32_len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn u32_is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn u32_remove(index: u64) -> u32 {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn u32_insert(index: u64, value: u32) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn u32_pop() -> u32 {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn u32_swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn u32_swap_remove(index: u64) -> u32 {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn u32_set(index: u64, value: u32) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn u32_first() -> u32 {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn u32_last() -> u32 {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn u32_reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn u32_fill(value: u32) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn u32_resize(new_len: u64, value: u32) {
        storage.my_vec.resize(new_len, value);
    }

    #[storage(read, write)]
    fn u32_append() {
        storage.my_vec.append(storage.my_other_vec);
    }
}
