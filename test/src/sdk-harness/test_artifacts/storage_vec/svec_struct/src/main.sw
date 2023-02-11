contract;

use std::storage::StorageVec;

struct TestStruct {
    A: bool,
    B: u16,
}

abi MyContract {
    #[storage(read, write)]
    fn struct_push(value: TestStruct);

    #[storage(read, write)]
    fn struct_push_other(value: TestStruct);

    #[storage(write)]
    fn struct_clear();
    #[storage(read)]
    fn struct_get(index: u64) -> TestStruct;

    #[storage(read)]
    fn struct_len() -> u64;

    #[storage(read)]
    fn struct_is_empty() -> bool;

    #[storage(read, write)]
    fn struct_remove(index: u64) -> TestStruct;

    #[storage(read, write)]
    fn struct_insert(index: u64, value: TestStruct);

    #[storage(read, write)]
    fn struct_pop() -> TestStruct;

    #[storage(read, write)]
    fn struct_swap(index_0: u64, index_1: u64);

    #[storage(read, write)]
    fn struct_swap_remove(index: u64) -> TestStruct;

    #[storage(read, write)]
    fn struct_set(index: u64, value: TestStruct);

    #[storage(read)]
    fn struct_first() -> TestStruct;

    #[storage(read)]
    fn struct_last() -> TestStruct;

    #[storage(read, write)]
    fn struct_reverse();

    #[storage(read, write)]
    fn struct_fill(value: TestStruct);

    #[storage(read, write)]
    fn struct_resize(new_len: u64, value: TestStruct);

    #[storage(read, write)]
    fn struct_append();
}

storage {
    my_vec: StorageVec<TestStruct> = StorageVec {},
    my_other_vec: StorageVec<TestStruct> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn struct_push(value: TestStruct) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn struct_push_other(value: TestStruct) {
        storage.my_other_vec.push(value);
    }

    #[storage(write)]
    fn struct_clear() {
        storage.my_vec.clear();
    }

    #[storage(read)]
    fn struct_get(index: u64) -> TestStruct {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read)]
    fn struct_len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn struct_is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(read, write)]
    fn struct_remove(index: u64) -> TestStruct {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn struct_insert(index: u64, value: TestStruct) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read, write)]
    fn struct_pop() -> TestStruct {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read, write)]
    fn struct_swap(index_0: u64, index_1: u64) {
        storage.my_vec.swap(index_0, index_1);
    }

    #[storage(read, write)]
    fn struct_swap_remove(index: u64) -> TestStruct {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn struct_set(index: u64, value: TestStruct) {
        storage.my_vec.set(index, value);
    }

    #[storage(read)]
    fn struct_first() -> TestStruct {
        storage.my_vec.first().unwrap()
    }

    #[storage(read)]
    fn struct_last() -> TestStruct {
        storage.my_vec.last().unwrap()
    }

    #[storage(read, write)]
    fn struct_reverse() {
        storage.my_vec.reverse();
    }

    #[storage(read, write)]
    fn struct_fill(value: TestStruct) {
        storage.my_vec.fill(value);
    }

    #[storage(read, write)]
    fn struct_resize(new_len: u64, value: TestStruct) {
        storage.my_vec.resize(new_len, value);
    }

    #[storage(read, write)]
    fn struct_append() {
        storage.my_vec.append(storage.my_other_vec);
    }
}
