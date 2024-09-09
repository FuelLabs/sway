contract;

use std::storage::storage_vec::*;

pub struct TestStruct {
    val1: u64,
    val2: u64,
    val3: u64,
}

impl Eq for TestStruct {
    fn eq(self, other: Self) -> bool {
        self.val1 == other.val1 && self.val2 == other.val2 && self.val3 == other.val3
    }
}

storage {
    storage_vec_u64: StorageVec<u64> = StorageVec {},
    storage_vec_struct: StorageVec<TestStruct> = StorageVec {},
    storage_vec_u8: StorageVec<u8> = StorageVec {},
}

abi VecToVecStorageTest {
    #[storage(read, write)]
    fn store_vec_u64(vec: Vec<u64>);
    #[storage(read)]
    fn read_vec_u64() -> Vec<u64>;
    #[storage(read, write)]
    fn push_vec_u64(val: u64);
    #[storage(read, write)]
    fn pop_vec_u64() -> u64;
    #[storage(read, write)]
    fn store_vec_struct(vec: Vec<TestStruct>);
    #[storage(read)]
    fn read_vec_struct() -> Vec<TestStruct>;
    #[storage(read, write)]
    fn push_vec_struct(val: TestStruct);
    #[storage(read, write)]
    fn pop_vec_struct() -> TestStruct;
    #[storage(read, write)]
    fn store_vec_u8(vec: Vec<u8>);
    #[storage(read)]
    fn read_vec_u8() -> Vec<u8>;
    #[storage(read, write)]
    fn push_vec_u8(val: u8);
    #[storage(read, write)]
    fn pop_vec_u8() -> u8;
}

impl VecToVecStorageTest for Contract {
    #[storage(read, write)]
    fn store_vec_u64(vec: Vec<u64>) {
        storage.storage_vec_u64.store_vec(vec);
    }

    #[storage(read)]
    fn read_vec_u64() -> Vec<u64> {
        storage.storage_vec_u64.load_vec()
    }

    #[storage(read, write)]
    fn push_vec_u64(val: u64) {
        storage.storage_vec_u64.push(val);
    }

    #[storage(read, write)]
    fn pop_vec_u64() -> u64 {
        storage.storage_vec_u64.pop().unwrap_or(0)
    }

    #[storage(read, write)]
    fn store_vec_struct(vec: Vec<TestStruct>) {
        storage.storage_vec_struct.store_vec(vec);
    }

    #[storage(read)]
    fn read_vec_struct() -> Vec<TestStruct> {
        storage.storage_vec_struct.load_vec()
    }

    #[storage(read, write)]
    fn push_vec_struct(val: TestStruct) {
        storage.storage_vec_struct.push(val);
    }

    #[storage(read, write)]
    fn pop_vec_struct() -> TestStruct {
        storage.storage_vec_struct.pop().unwrap_or(TestStruct {
            val1: 0,
            val2: 0,
            val3: 0,
        })
    }

    #[storage(read, write)]
    fn store_vec_u8(vec: Vec<u8>) {
        storage.storage_vec_u8.store_vec(vec);
    }

    #[storage(read)]
    fn read_vec_u8() -> Vec<u8> {
        storage.storage_vec_u8.load_vec()
    }

    #[storage(read, write)]
    fn push_vec_u8(val: u8) {
        storage.storage_vec_u8.push(val);
    }

    #[storage(read, write)]
    fn pop_vec_u8() -> u8 {
        storage.storage_vec_u8.pop().unwrap_or(0)
    }
}
