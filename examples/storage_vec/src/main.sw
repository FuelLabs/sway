contract;

// ANCHOR: storage_vec_import
use std::storage::storage_vec::*;
// ANCHOR_END: storage_vec_import
// ANCHOR: storage_vec_multiple_types_enum
enum TableCell {
    Int: u64,
    B256: b256,
    Boolean: bool,
}
// ANCHOR_END: storage_vec_multiple_types_enum
storage {
    // ANCHOR: storage_vec_decl
    v: StorageVec<u64> = StorageVec {},
    // ANCHOR_END: storage_vec_decl
    // ANCHOR: storage_vec_multiple_types_decl
    row: StorageVec<TableCell> = StorageVec {},
    // ANCHOR_END: storage_vec_multiple_types_decl
    // ANCHOR: storage_vec_nested
    nested_vec: StorageVec<StorageVec<u64>> = StorageVec {},
    // ANCHOR_END: storage_vec_nested
}

abi StorageVecContract {
    #[storage(read, write)]
    fn push_to_storage_vec();

    #[storage(read)]
    fn read_from_storage_vec();

    #[storage(read)]
    fn iterate_over_a_storage_vec();

    #[storage(read, write)]
    fn push_to_multiple_types_storage_vec();

    #[storage(read, write)]
    fn access_nested_vec();
}

impl StorageVecContract for Contract {
    // ANCHOR: storage_vec_push
    #[storage(read, write)]
    fn push_to_storage_vec() {
        storage.v.push(5);
        storage.v.push(6);
        storage.v.push(7);
        storage.v.push(8);
    }
    // ANCHOR_END: storage_vec_push
    // ANCHOR: storage_vec_get
    #[storage(read)]
    fn read_from_storage_vec() {
        let third = storage.v.get(2);
        match third {
            Some(third) => log(third.read()),
            None => revert(42),
        }
    }
    // ANCHOR_END: storage_vec_get
    // ANCHOR: storage_vec_iterate
    #[storage(read)]
    fn iterate_over_a_storage_vec() {
        let mut i = 0;
        while i < storage.v.len() {
            log(storage.v.get(i).unwrap().read());
            i += 1;
        }
    }
    // ANCHOR_END: storage_vec_iterate
    // ANCHOR: storage_vec_multiple_types_fn
    #[storage(read, write)]
    fn push_to_multiple_types_storage_vec() {
        storage.row.push(TableCell::Int(3));
        storage.row.push(TableCell::B256(0x0101010101010101010101010101010101010101010101010101010101010101));
        storage.row.push(TableCell::Boolean(true));
    }
    // ANCHOR_END: storage_vec_multiple_types_fn

    // ANCHOR: access_nested_vec 
    #[storage(read, write)]
    fn access_nested_vec() {
        storage.nested_vec.push(StorageVec {});
        storage.nested_vec.push(StorageVec {});

        let mut inner_vec0 = storage.nested_vec.get(0).unwrap();
        let mut inner_vec1 = storage.nested_vec.get(1).unwrap();

        inner_vec0.push(0);
        inner_vec0.push(1);

        inner_vec1.push(2);
        inner_vec1.push(3);
        inner_vec1.push(4);

        assert(inner_vec0.len() == 2);
        assert(inner_vec0.get(0).unwrap().read() == 0);
        assert(inner_vec0.get(1).unwrap().read() == 1);
        assert(inner_vec0.get(2).is_none());

        assert(inner_vec1.len() == 3);
        assert(inner_vec1.get(0).unwrap().read() == 2);
        assert(inner_vec1.get(1).unwrap().read() == 3);
        assert(inner_vec1.get(2).unwrap().read() == 4);
        assert(inner_vec1.get(3).is_none());
    }
    // ANCHOR_END: access_nested_vec 
}
