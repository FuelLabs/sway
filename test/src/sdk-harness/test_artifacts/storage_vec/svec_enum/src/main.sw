contract;

use std::storage::StorageVec;

enum TestEnum {
    A: bool,
    B: u16,
}

abi MyContract {
    #[storage(read, write)]
    fn enum_push(value: TestEnum);

    #[storage(read, write)]
    fn enum_push_other(value: TestEnum);

    #[storage(write)]
    fn enum_clear();
    #[storage(read)]
    fn enum_get(index: u64) -> TestEnum;

    #[storage(read)]
    fn enum_len() -> u64;

    #[storage(read)]
    fn enum_is_empty() -> bool;

    #[storage(read, write)]
    fn enum_remove(index: u64) -> TestEnum;

    #[storage(read, write)]
    fn enum_insert(index: u64, value: TestEnum);

    #[storage(read, write)]
    fn enum_pop() -> TestEnum;

    #[storage(read, write)]
    fn enum_swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn enum_swap_remove(index: u64) -> TestEnum;

    #[storage(read, write)]
    fn enum_set(index: u64, value: TestEnum);

    #[storage(read)]
    fn enum_first() -> TestEnum;

    #[storage(read)]
    fn enum_last() -> TestEnum;

    #[storage(read, write)]
    fn enum_reverse();

    #[storage(read, write)]
    fn enum_fill(value: TestEnum);

    #[storage(read, write)]
    fn enum_resize(new_len: u64, value: TestEnum);

    #[storage(read, write)]
    fn enum_append();
}

storage {
    my_vec: StorageVec<TestEnum> = StorageVec {},
    my_other_vec: StorageVec<TestEnum> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn enum_push(value: TestEnum) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn enum_push_other(value: TestEnum) {
        storage.my_other_vec.push(value);
    }

    #[storage(write)]
    fn enum_clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn enum_get(index: u64) -> TestEnum {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn enum_len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn enum_is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn enum_remove(index: u64) -> TestEnum {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn enum_insert(index: u64, value: TestEnum) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn enum_pop() -> TestEnum {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn enum_swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn enum_swap_remove(index: u64) -> TestEnum {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn enum_set(index: u64, value: TestEnum) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn enum_first() -> TestEnum {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn enum_last() -> TestEnum {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn enum_reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn enum_fill(value: TestEnum) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn enum_resize(new_len: u64, value: TestEnum) {
        storage.my_vec.resize(new_len, value);
    }

    #[storage(read, write)]
    fn enum_append() {
        storage.my_vec.append(storage.my_other_vec);
    }
}
