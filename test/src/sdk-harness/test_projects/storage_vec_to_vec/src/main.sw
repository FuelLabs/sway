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
}

#[test]
fn test_conversion_u64() {
    let vec_abi = abi(VecToVecStorageTest, CONTRACT_ID);
    let mut test_vec = Vec::<u64>::new();
    test_vec.push(5);
    test_vec.push(7);
    test_vec.push(9);
    test_vec.push(11);

    vec_abi.store_vec_u64(test_vec);

    let returned_vec = vec_abi.read_vec_u64();

    assert(returned_vec.len() == 4);
    assert(returned_vec.get(0).unwrap() == 5);
    assert(returned_vec.get(1).unwrap() == 7);
    assert(returned_vec.get(2).unwrap() == 9);
    assert(returned_vec.get(3).unwrap() == 11);
}

#[test]
fn test_push_u64() {
    let vec_abi = abi(VecToVecStorageTest, CONTRACT_ID);
    let mut test_vec = Vec::<u64>::new();
    test_vec.push(5);
    test_vec.push(7);
    test_vec.push(9);
    test_vec.push(11);

    vec_abi.store_vec_u64(test_vec);

    vec_abi.push_vec_u64(13);

    let returned_vec = vec_abi.read_vec_u64();

    assert(returned_vec.len() == 5);
    assert(returned_vec.get(0).unwrap() == 5);
    assert(returned_vec.get(1).unwrap() == 7);
    assert(returned_vec.get(2).unwrap() == 9);
    assert(returned_vec.get(3).unwrap() == 11);
    assert(returned_vec.get(4).unwrap() == 13);
}

#[test]
fn test_pop_u64() {
    let vec_abi = abi(VecToVecStorageTest, CONTRACT_ID);
    let mut test_vec = Vec::<u64>::new();
    test_vec.push(5);
    test_vec.push(7);
    test_vec.push(9);
    test_vec.push(11);

    vec_abi.store_vec_u64(test_vec);

    assert(11 == vec_abi.pop_vec_u64());

    let returned_vec = vec_abi.read_vec_u64();

    assert(returned_vec.len() == 3);
    assert(returned_vec.get(0).unwrap() == 5);
    assert(returned_vec.get(1).unwrap() == 7);
    assert(returned_vec.get(2).unwrap() == 9);
}

#[test]
fn test_conversion_struct() {
    let vec_abi = abi(VecToVecStorageTest, CONTRACT_ID);
    let mut test_vec = Vec::<TestStruct>::new();
    test_vec.push(TestStruct {
        val1: 0,
        val2: 1,
        val3: 2,
    });
    test_vec.push(TestStruct {
        val1: 1,
        val2: 2,
        val3: 3,
    });
    test_vec.push(TestStruct {
        val1: 2,
        val2: 3,
        val3: 4,
    });
    test_vec.push(TestStruct {
        val1: 3,
        val2: 4,
        val3: 5,
    });

    vec_abi.store_vec_struct(test_vec);

    let returned_vec = vec_abi.read_vec_struct();

    assert(returned_vec.len() == 4);
    assert(
        returned_vec
            .get(0)
            .unwrap() == TestStruct {
            val1: 0,
            val2: 1,
            val3: 2,
        },
    );
    assert(
        returned_vec
            .get(1)
            .unwrap() == TestStruct {
            val1: 1,
            val2: 2,
            val3: 3,
        },
    );
    assert(
        returned_vec
            .get(2)
            .unwrap() == TestStruct {
            val1: 2,
            val2: 3,
            val3: 4,
        },
    );
    assert(
        returned_vec
            .get(3)
            .unwrap() == TestStruct {
            val1: 3,
            val2: 4,
            val3: 5,
        },
    );
}

#[test]
fn test_push_struct() {
    let vec_abi = abi(VecToVecStorageTest, CONTRACT_ID);
    let mut test_vec = Vec::<TestStruct>::new();
    test_vec.push(TestStruct {
        val1: 0,
        val2: 1,
        val3: 2,
    });
    test_vec.push(TestStruct {
        val1: 1,
        val2: 2,
        val3: 3,
    });
    test_vec.push(TestStruct {
        val1: 2,
        val2: 3,
        val3: 4,
    });
    test_vec.push(TestStruct {
        val1: 3,
        val2: 4,
        val3: 5,
    });

    vec_abi.store_vec_struct(test_vec);

    vec_abi.push_vec_struct(TestStruct {
        val1: 4,
        val2: 5,
        val3: 6,
    });

    let returned_vec = vec_abi.read_vec_struct();

    assert(returned_vec.len() == 5);
    assert(
        returned_vec
            .get(0)
            .unwrap() == TestStruct {
            val1: 0,
            val2: 1,
            val3: 2,
        },
    );
    assert(
        returned_vec
            .get(1)
            .unwrap() == TestStruct {
            val1: 1,
            val2: 2,
            val3: 3,
        },
    );
    assert(
        returned_vec
            .get(2)
            .unwrap() == TestStruct {
            val1: 2,
            val2: 3,
            val3: 4,
        },
    );
    assert(
        returned_vec
            .get(3)
            .unwrap() == TestStruct {
            val1: 3,
            val2: 4,
            val3: 5,
        },
    );
    assert(
        returned_vec
            .get(4)
            .unwrap() == TestStruct {
            val1: 4,
            val2: 5,
            val3: 6,
        },
    );
}

#[test]
fn test_pop_struct() {
    let vec_abi = abi(VecToVecStorageTest, CONTRACT_ID);
    let mut test_vec = Vec::<TestStruct>::new();
    test_vec.push(TestStruct {
        val1: 0,
        val2: 1,
        val3: 2,
    });
    test_vec.push(TestStruct {
        val1: 1,
        val2: 2,
        val3: 3,
    });
    test_vec.push(TestStruct {
        val1: 2,
        val2: 3,
        val3: 4,
    });
    test_vec.push(TestStruct {
        val1: 3,
        val2: 4,
        val3: 5,
    });

    vec_abi.store_vec_struct(test_vec);

    assert(
        TestStruct {
            val1: 3,
            val2: 4,
            val3: 5,
        } == vec_abi
            .pop_vec_struct(),
    );

    let returned_vec = vec_abi.read_vec_struct();

    assert(returned_vec.len() == 3);
    assert(
        returned_vec
            .get(0)
            .unwrap() == TestStruct {
            val1: 0,
            val2: 1,
            val3: 2,
        },
    );
    assert(
        returned_vec
            .get(1)
            .unwrap() == TestStruct {
            val1: 1,
            val2: 2,
            val3: 3,
        },
    );
    assert(
        returned_vec
            .get(2)
            .unwrap() == TestStruct {
            val1: 2,
            val2: 3,
            val3: 4,
        },
    );
}
