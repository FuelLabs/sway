contract;

use std::mapping::*;

storage {
    mapping1: Mapping<u64, u64>,
    mapping2: Mapping<b256, bool>,
    mapping3: Mapping<(u64, bool), b256>,
}

abi MappingTest {
    fn init();

    fn insert_into_mapping1(key: u64, value: u64 );
    fn get_from_mapping1(key: u64) -> u64;   

    fn insert_into_mapping2(key: b256, value: bool );
    fn get_from_mapping2(key: b256) -> bool;   

    fn insert_into_mapping3(key: (u64, bool), value: b256 );
    fn get_from_mapping3(key: (u64, bool)) -> b256;   
}

impl MappingTest for Contract {
    fn init() {
        storage.mapping1 = ~Mapping::new::<u64, u64>();
        storage.mapping2 = ~Mapping::new::<b256, bool>();
    }

    fn insert_into_mapping1(key: u64, value: u64 ) {
        storage.mapping1.insert(key, value);
    }

    fn get_from_mapping1(key: u64) -> u64 {
        storage.mapping1.get(key)
    }   

    fn insert_into_mapping2(key: b256, value: bool ) {
        storage.mapping2.insert(key, value);
    }

    fn get_from_mapping2(key: b256) -> bool {
        storage.mapping2.get(key)
    }

    fn insert_into_mapping3(key: (u64, bool), value: b256 ) {
        storage.mapping3.insert(key, value);
    }

    fn get_from_mapping3(key: (u64, bool)) -> b256 {
        storage.mapping3.get(key)
    }
}
