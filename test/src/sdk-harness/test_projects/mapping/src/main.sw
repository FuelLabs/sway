contract;

use std::mapping::Mapping;

storage {
    mapping1: Mapping,
    mapping2: Mapping
}

abi MyContract {
    fn init();

    fn insert_into_mapping1(key: u64, value: u64 );
    fn get_from_mapping1(key: u64) -> u64;

    fn insert_into_mapping2(key: u64, value: u64 );
    fn get_from_mapping2(key: u64) -> u64;
}

impl MyContract for Contract {
    fn init() {
        storage.mapping1 = ~Mapping::new();
        storage.mapping2 = ~Mapping::new();
    }

    fn insert_into_mapping1(key: u64, value: u64 ) {
        storage.mapping1.insert(key, value);
    }

    fn get_from_mapping1(key: u64) -> u64 {
        storage.mapping1.get(key)
    }

    fn insert_into_mapping2(key: u64, value: u64 ) {
        storage.mapping2.insert(key, value);
    }

    fn get_from_mapping2(key: u64) -> u64 {
        storage.mapping2.get(key)
    }
}
