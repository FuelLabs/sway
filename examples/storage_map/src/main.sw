contract;

// ANCHOR: storage_map_import
use std::storage::StorageMap;
// ANCHOR_END: storage_map_import
use std::{address::Address, logging::log, option::Option, revert::revert};

storage {
    // ANCHOR: storage_map_decl
    map: StorageMap<Address, u64> = StorageMap {},
    // ANCHOR_END: storage_map_decl
    // ANCHOR: storage_map_tuple_key
    map_two_keys: StorageMap<(b256, bool), b256> = StorageMap {},
    // ANCHOR_END: storage_map_tuple_key
}

abi StorageMapExample {
    #[storage(write)]
    fn insert_into_storage_map();

    #[storage(read, write)]
    fn get_from_storage_map();
}

impl StorageMapExample for Contract {
    // ANCHOR: storage_map_insert
    #[storage(write)]
    fn insert_into_storage_map() {
        let addr1 = 0x0101010101010101010101010101010101010101010101010101010101010101;
        let addr2 = 0x0202020202020202020202020202020202020202020202020202020202020202;

        storage.map.insert(~Address::from(addr1), 42);
        storage.map.insert(~Address::from(addr2), 77);
    }
    // ANCHOR_END: storage_map_insert

    // ANCHOR: storage_map_get
    #[storage(read, write)]
    fn get_from_storage_map() {
        let addr1 = 0x0101010101010101010101010101010101010101010101010101010101010101;
        let addr2 = 0x0202020202020202020202020202020202020202020202020202020202020202;

        storage.map.insert(~Address::from(addr1), 42);
        storage.map.insert(~Address::from(addr2), 77);

        let value1 = storage.map.get(~Address::from(addr1));
    }
    // ANCHOR_END: storage_map_get
}
