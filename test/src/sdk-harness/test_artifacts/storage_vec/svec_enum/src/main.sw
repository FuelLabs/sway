contract;

use std::storage::StorageVec;

enum TestEnum {
    A: bool,
    B: u16,
}

abi MyContract {
    #[storage(read, write)]
    fn enum_push(value: TestEnum);
    #[storage(read)]
    fn enum_get(index: u64) -> TestEnum;
    #[storage(read, write)]
    fn enum_pop() -> TestEnum;
    #[storage(read, write)]
    fn enum_remove(index: u64) -> TestEnum;
    #[storage(read, write)]
    fn enum_swap_remove(index: u64) -> TestEnum;
    #[storage(read, write)]
    fn enum_set(index: u64, value: TestEnum);
    #[storage(read, write)]
    fn enum_insert(index: u64, value: TestEnum);
    #[storage(read)]
    fn enum_len() -> u64;
    #[storage(read)]
    fn enum_is_empty() -> bool;
    #[storage(write)]
    fn enum_clear();
}

storage {
    my_vec: StorageVec<TestEnum> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn enum_push(value: TestEnum) {
        storage.my_vec.push(value);
    }
    #[storage(read)]
    fn enum_get(index: u64) -> TestEnum {
        storage.my_vec.get(index).unwrap()
    }
    #[storage(read, write)]
    fn enum_pop() -> TestEnum {
        storage.my_vec.pop().unwrap()
    }
    #[storage(read, write)]
    fn enum_remove(index: u64) -> TestEnum {
        storage.my_vec.remove(index)
    }
    #[storage(read, write)]
    fn enum_swap_remove(index: u64) -> TestEnum {
        storage.my_vec.swap_remove(index)
    }
    #[storage(read, write)]
    fn enum_set(index: u64, value: TestEnum) {
        storage.my_vec.set(index, value);
    }
    #[storage(read, write)]
    fn enum_insert(index: u64, value: TestEnum) {
        storage.my_vec.insert(index, value);
    }
    #[storage(read)]
    fn enum_len() -> u64 {
        storage.my_vec.len()
    }
    #[storage(read)]
    fn enum_is_empty() -> bool {
        storage.my_vec.is_empty()
    }
    #[storage(write)]
    fn enum_clear() {
        storage.my_vec.clear();
    }
}
