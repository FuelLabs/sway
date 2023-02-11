contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn u8_push(value: u8);

    #[storage(read, write)]
    fn u8_push_other(value: u8);

    #[storage(write)]
    fn u8_clear();
    #[storage(read)]
    fn u8_get(index: u64) -> u8;

    #[storage(read)]
    fn u8_len() -> u64;

    #[storage(read)]
    fn u8_is_empty() -> bool;

    #[storage(read, write)]
    fn u8_remove(index: u64) -> u8;

    #[storage(read, write)]
    fn u8_insert(index: u64, value: u8);

    #[storage(read, write)]
    fn u8_pop() -> u8;

    #[storage(read, write)]
    fn u8_swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn u8_swap_remove(index: u64) -> u8;

    #[storage(read, write)]
    fn u8_set(index: u64, value: u8);

    #[storage(read)]
    fn u8_first() -> u8;

    #[storage(read)]
    fn u8_last() -> u8;

    #[storage(read, write)]
    fn u8_reverse();

    #[storage(read, write)]
    fn u8_fill(value: u8);

    #[storage(read, write)]
    fn u8_resize(new_len: u64, value: u8);

    #[storage(read, write)]
    fn u8_append();
}

storage {
    my_vec: StorageVec<u8> = StorageVec {},
    my_other_vec: StorageVec<u8> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn u8_push(value: u8) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn u8_push_other(value: u8) {
        storage.my_other_vec.push(value);
    }

    #[storage(write)]
    fn u8_clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn u8_get(index: u64) -> u8 {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn u8_len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn u8_is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn u8_remove(index: u64) -> u8 {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn u8_insert(index: u64, value: u8) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn u8_pop() -> u8 {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn u8_swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn u8_swap_remove(index: u64) -> u8 {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn u8_set(index: u64, value: u8) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn u8_first() -> u8 {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn u8_last() -> u8 {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn u8_reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn u8_fill(value: u8) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn u8_resize(new_len: u64, value: u8) {
        storage.my_vec.resize(new_len, value);
    }

    #[storage(read, write)]
    fn u8_append() {
        storage.my_vec.append(storage.my_other_vec);
    }
}
