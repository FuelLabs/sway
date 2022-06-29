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
    #[storage(write)]fn insert_into_map1(key: u64, value: u64);

    #[storage(read)]fn get_from_map1(key: u64, value: u64) -> u64;

    #[storage(write)]fn insert_into_map2(key: (b256, bool), value: Data);

    #[storage(read)]fn get_from_map2(key: (b256, bool), value: Data) -> Data;
}

impl StorageMapExample for Contract {
    #[storage(write)]fn insert_into_map1(key: u64, value: u64) {
        storage.map1.insert(key, value);
    }

    #[storage(read)]fn get_from_map1(key: u64, value: u64) -> u64 {
        storage.map1.get(key)
    }

    #[storage(write)]fn insert_into_map2(key: (b256, bool), value: Data) {
        storage.map2.insert(key, value);
    }

    #[storage(read)]fn get_from_map2(key: (b256, bool), value: Data) -> Data {
        storage.map2.get(key)
    }
}
