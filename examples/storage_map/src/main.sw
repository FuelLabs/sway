contract;

use std::logging::log;

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
        let addr1 = Address::from(0x0101010101010101010101010101010101010101010101010101010101010101);
        let addr2 = Address::from(0x0202020202020202020202020202020202020202020202020202020202020202);

        storage.map.insert(addr1, 42);
        storage.map.insert(addr2, 77);
    }
    // ANCHOR_END: storage_map_insert
    // ANCHOR: storage_map_get
    #[storage(read, write)]
    fn get_from_storage_map() {
        let addr1 = Address::from(0x0101010101010101010101010101010101010101010101010101010101010101);
        let addr2 = Address::from(0x0202020202020202020202020202020202020202020202020202020202020202);

        storage.map.insert(addr1, 42);
        storage.map.insert(addr2, 77);

        let value1 = storage.map.get(addr1);
    }
    // ANCHOR_END: storage_map_get
}
