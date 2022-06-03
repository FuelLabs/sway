contract;

use std::storage::StorageMap;

struct Data {
    x: b256,
    y: str[4],
}

storage {
    map1: StorageMap<u64,
    u64>, map2: StorageMap<(b256,
    bool), Data>, 
}

abi StorageMapExample {
    fn insert_into_map1(key: u64, value: u64);

    fn get_from_map1(key: u64, value: u64);

    fn insert_into_map2(key: (b256, bool), value: Data);

    fn get_from_map2(key: (b256, bool), value: Data);
}

impl StorageMapExample for Contract {
    fn insert_into_map1(key: u64, value: u64) {
        storage.map1.insert(key, value);
    }

    fn get_from_map1(key: u64, value: u64) {
        storage.map1.insert(key, value);
    }

    fn insert_into_map2(key: (b256, bool), value: Data) {
        storage.map2.get(key);
    }

    fn get_from_map2(key: (b256, bool), value: Data) {
        storage.map2.get(key);
    }
}
