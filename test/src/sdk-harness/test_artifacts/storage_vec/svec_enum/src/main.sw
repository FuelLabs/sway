contract;

use std::storage::StorageVec;

enum TestEnum {
    A: bool,
    B: u16,
}

abi MyContract {
    #[storage(read, write)]
    fn push(value: TestEnum);

    #[storage(write)]
    fn clear();

    #[storage(read)]
    fn get(index: u64) -> TestEnum;

    #[storage(read)]
    fn len() -> u64;

    #[storage(read)]
    fn is_empty() -> bool;

    #[storage(read, write)]
    fn remove(index: u64) -> TestEnum;

    #[storage(read, write)]
    fn insert(index: u64, value: TestEnum);

    #[storage(read, write)]
    fn pop() -> TestEnum;

    #[storage(read, write)]
    fn swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn swap_remove(index: u64) -> TestEnum;

    #[storage(read, write)]
    fn set(index: u64, value: TestEnum);

    #[storage(read)]
    fn first() -> TestEnum;

    #[storage(read)]
    fn last() -> TestEnum;

    #[storage(read, write)]
    fn reverse();

    #[storage(read, write)]
    fn fill(value: TestEnum);

    #[storage(read, write)]
    fn resize(new_len: u64, value: TestEnum);
}

storage {
    my_vec: StorageVec<TestEnum> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn push(value: TestEnum) {
        storage.my_vec.push(value);
    }

    #[storage(write)]
    fn clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn get(index: u64) -> TestEnum {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn remove(index: u64) -> TestEnum {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn insert(index: u64, value: TestEnum) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn pop() -> TestEnum {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn swap_remove(index: u64) -> TestEnum {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn set(index: u64, value: TestEnum) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn first() -> TestEnum {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn last() -> TestEnum {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn fill(value: TestEnum) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn resize(new_len: u64, value: TestEnum) {
        storage.my_vec.resize(new_len, value);
    }
}
