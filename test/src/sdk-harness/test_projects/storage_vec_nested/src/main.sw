contract;

use std::storage::storage_vec::*;

storage {
    nested_vec: StorageVec<StorageVec<u64>> = StorageVec {},
}

abi ExperimentalStorageTest {
    #[storage(read, write)]
    fn nested_vec_access();
}

impl ExperimentalStorageTest for Contract {
    #[storage(read, write)]
    fn nested_vec_access() {
        storage.nested_vec.push(StorageVec {});
        storage.nested_vec.push(StorageVec {});
        storage.nested_vec.push(StorageVec {});
        let mut inner_vec0 = storage.nested_vec.get(0).unwrap();
        let mut inner_vec1 = storage.nested_vec.get(1).unwrap();
        let mut inner_vec2 = storage.nested_vec.get(2).unwrap();
        assert(storage.nested_vec.len() == 3);
        assert(storage.nested_vec.get(3).is_none());

        inner_vec0.push(0);
        inner_vec0.push(1);

        inner_vec1.push(2);
        inner_vec1.push(3);
        inner_vec1.push(4);

        inner_vec2.push(5);
        inner_vec2.push(6);
        inner_vec2.push(7);
        inner_vec2.push(8);

        inner_vec0.set(0, 0);
        inner_vec0.set(1, 11);

        inner_vec1.set(0, 22);
        inner_vec1.set(1, 33);
        inner_vec1.set(2, 44);

        inner_vec2.set(0, 55);
        inner_vec2.set(1, 66);
        inner_vec2.set(2, 77);
        inner_vec2.set(3, 88);

        assert(inner_vec0.len() == 2);
        assert(inner_vec0.get(0).unwrap().read() == 0);
        assert(inner_vec0.get(1).unwrap().read() == 11);
        assert(inner_vec0.get(2).is_none());

        assert(inner_vec1.len() == 3);
        assert(inner_vec1.get(0).unwrap().read() == 22);
        assert(inner_vec1.get(1).unwrap().read() == 33);
        assert(inner_vec1.get(2).unwrap().read() == 44);
        assert(inner_vec1.get(3).is_none());

        assert(inner_vec2.len() == 4);
        assert(inner_vec2.get(0).unwrap().read() == 55);
        assert(inner_vec2.get(1).unwrap().read() == 66);
        assert(inner_vec2.get(2).unwrap().read() == 77);
        assert(inner_vec2.get(3).unwrap().read() == 88);
        assert(inner_vec2.get(4).is_none());
    }
}
