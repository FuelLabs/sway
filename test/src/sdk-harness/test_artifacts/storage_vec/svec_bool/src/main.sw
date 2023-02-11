contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn bool_push(value: bool);

    #[storage(read, write)]
    fn bool_push_other(value: bool);

    #[storage(write)]
    fn bool_clear();
    #[storage(read)]
    fn bool_get(index: u64) -> bool;

    #[storage(read)]
    fn bool_len() -> u64;

    #[storage(read)]
    fn bool_is_empty() -> bool;

    #[storage(read, write)]
    fn bool_remove(index: u64) -> bool;

    #[storage(read, write)]
    fn bool_insert(index: u64, value: bool);

    #[storage(read, write)]
    fn bool_pop() -> bool;

    #[storage(read, write)]
    fn bool_swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn bool_swap_remove(index: u64) -> bool;

    #[storage(read, write)]
    fn bool_set(index: u64, value: bool);

    #[storage(read)]
    fn bool_first() -> bool;

    #[storage(read)]
    fn bool_last() -> bool;

    #[storage(read, write)]
    fn bool_reverse();

    #[storage(read, write)]
    fn bool_fill(value: bool);

    #[storage(read, write)]
    fn bool_resize(new_len: u64, value: bool);

    #[storage(read, write)]
    fn bool_append();
}

storage {
    my_vec: StorageVec<bool> = StorageVec {},
    my_other_vec: StorageVec<bool> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn bool_push(value: bool) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn bool_push_other(value: bool) {
        storage.my_other_vec.push(value);
    }

    #[storage(write)]
    fn bool_clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn bool_get(index: u64) -> bool {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn bool_len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn bool_is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn bool_remove(index: u64) -> bool {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn bool_insert(index: u64, value: bool) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn bool_pop() -> bool {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn bool_swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn bool_swap_remove(index: u64) -> bool {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn bool_set(index: u64, value: bool) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn bool_first() -> bool {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn bool_last() -> bool {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn bool_reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn bool_fill(value: bool) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn bool_resize(new_len: u64, value: bool) {
        storage.my_vec.resize(new_len, value);
    }

    #[storage(read, write)]
    fn bool_append() {
        storage.my_vec.append(storage.my_other_vec);
    }
}
