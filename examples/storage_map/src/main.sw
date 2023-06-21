contract;

use std::hash::*;

impl Hash for (b256, bool) {
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

storage {
    // ANCHOR: storage_map_decl
    map: StorageMap<Address, u64> = StorageMap::<Address, u64> {},
    // ANCHOR_END: storage_map_decl
    // ANCHOR: storage_map_tuple_key
    map_two_keys: StorageMap<(b256, bool), b256> = StorageMap::<(b256, bool), b256> {},
    // ANCHOR_END: storage_map_tuple_key
    // ANCHOR: storage_map_nested
    nested_map: StorageMap<u64, StorageMap<u64, u64>> = StorageMap::<u64, StorageMap<u64, u64>> {},
    // ANCHOR_END: storage_map_nested
}

abi StorageMapExample {
    #[storage(write)]
    fn insert_into_storage_map();

    #[storage(read, write)]
    fn get_from_storage_map();

    #[storage(read, write)]
    fn access_nested_map();
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

        let value1 = storage.map.get(addr1).try_read().unwrap_or(0);
    }
    // ANCHOR_END: storage_map_get

    // ANCHOR: storage_map_nested_access
    #[storage(read, write)]
    fn access_nested_map() {
        storage.nested_map.get(0).insert(1, 42);
        storage.nested_map.get(2).insert(3, 24);

        assert(storage.nested_map.get(0).get(1).read() == 42);
        assert(storage.nested_map.get(0).get(0).try_read().is_none()); // Nothing inserted here
        assert(storage.nested_map.get(2).get(3).read() == 24);
        assert(storage.nested_map.get(2).get(2).try_read().is_none()); // Nothing inserted here
    }
    // ANCHOR_END: storage_map_nested_access
}
