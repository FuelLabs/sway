contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn b256_push(value: b256);

    #[storage(read, write)]
    fn b256_push_other(value: b256);

    #[storage(write)]
    fn b256_clear();
    #[storage(read)]
    fn b256_get(index: u64) -> b256;

    #[storage(read)]
    fn b256_len() -> u64;

    #[storage(read)]
    fn b256_is_empty() -> bool;

    #[storage(read, write)]
    fn b256_remove(index: u64) -> b256;

    #[storage(read, write)]
    fn b256_insert(index: u64, value: b256);

    #[storage(read, write)]
    fn b256_pop() -> b256;

    #[storage(read, write)]
    fn b256_swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn b256_swap_remove(index: u64) -> b256;

    #[storage(read, write)]
    fn b256_set(index: u64, value: b256);

    #[storage(read)]
    fn b256_first() -> b256;

    #[storage(read)]
    fn b256_last() -> b256;

    #[storage(read, write)]
    fn b256_reverse();

    #[storage(read, write)]
    fn b256_fill(value: b256);

    #[storage(read, write)]
    fn b256_resize(new_len: u64, value: b256);

    #[storage(read, write)]
    fn b256_append();
}

storage {
    my_vec: StorageVec<b256> = StorageVec {},
    my_other_vec: StorageVec<b256> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn b256_push(value: b256) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn b256_push_other(value: b256) {
        storage.my_other_vec.push(value);
    }

    #[storage(write)]
    fn b256_clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn b256_get(index: u64) -> b256 {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn b256_len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn b256_is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn b256_remove(index: u64) -> b256 {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn b256_insert(index: u64, value: b256) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn b256_pop() -> b256 {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn b256_swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn b256_swap_remove(index: u64) -> b256 {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn b256_set(index: u64, value: b256) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn b256_first() -> b256 {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn b256_last() -> b256 {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn b256_reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn b256_fill(value: b256) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn b256_resize(new_len: u64, value: b256) {
        storage.my_vec.resize(new_len, value);
    }

    #[storage(read, write)]
    fn b256_append() {
        storage.my_vec.append(storage.my_other_vec);
    }
}
