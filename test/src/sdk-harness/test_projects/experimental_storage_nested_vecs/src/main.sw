contract;

use core::experimental::storage::*;
use std::experimental::storage::*;

storage {
    nested_vec_1: StorageVec<StorageVec<u64>> = StorageVec{ },
}

abi ExperimentalStorageTest {
    #[storage(read, write)]
    fn nested_vec_1_access();
}

impl ExperimentalStorageTest for Contract {
    #[storage(read, write)]
    fn nested_vec_1_access() {
        storage.nested_vec_1.push(StorageVec { });
        storage.nested_vec_1.push(StorageVec { });
        storage.nested_vec_1.push(StorageVec { });
        let mut inner_vec1 = storage.nested_vec_1[0];
        let mut inner_vec2 = storage.nested_vec_1[1];
        let mut inner_vec3 = storage.nested_vec_1[2];
        assert(storage.nested_vec_1.len() == 3);
        assert(storage.nested_vec_1.get(3).is_none());
    
        inner_vec1.push(0);
        inner_vec1.push(1);

        inner_vec2.push(2);
        inner_vec2.push(3);
        inner_vec2.push(4);

        inner_vec3.push(5);
        inner_vec3.push(6);
        inner_vec3.push(7);
        inner_vec3.push(8);

        // Test `[]`
        // inner_vec1[0] = 0; // this line does not work yet - the fix is simple though.

        storage.nested_vec_1[0][0] = 0;
        storage.nested_vec_1[0][1] = 11;

        storage.nested_vec_1[1][0] = 22;
        storage.nested_vec_1[1][1] = 33;
        storage.nested_vec_1[1][2] = 44;

        storage.nested_vec_1[2][0] = 55;
        storage.nested_vec_1[2][1] = 66;
        storage.nested_vec_1[2][2] = 77;
        storage.nested_vec_1[2][3] = 88;

        assert(inner_vec1.len() == 2);
        assert(inner_vec1[0].read() == 0);
        assert(inner_vec1[1].read() == 11);
        assert(inner_vec1.get(2).is_none());

        assert(inner_vec2.len() == 3);
        assert(inner_vec2[0].read() == 22);
        assert(inner_vec2[1].read() == 33);
        assert(inner_vec2[2].read() == 44);
        assert(inner_vec2.get(3).is_none());

        assert(inner_vec3.len() == 4);
        assert(inner_vec3[0].read() == 55);
        assert(inner_vec3[1].read() == 66);
        assert(inner_vec3[2].read() == 77);
        assert(inner_vec3[3].read() == 88);
        assert(inner_vec3.get(4).is_none());
    }
}
