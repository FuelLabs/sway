contract;

// ANCHOR: storage_vec_import
use std::storage::StorageVec;
// ANCHOR_END: storage_vec_import
use std::{logging::log, option::Option};

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
            Option::Some(third) => log(third),
            Option::None => revert(42),
        }
    }
    // ANCHOR_END: storage_vec_get
    // ANCHOR: storage_vec_iterate
    #[storage(read)]
    fn iterate_over_a_storage_vec() {
        let mut i = 0;
        while i < storage.v.len() {
            log(storage.v.get(i).unwrap());
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
}
