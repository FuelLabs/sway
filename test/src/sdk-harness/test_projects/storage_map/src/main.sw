contract;

use std::storage::StorageMap;

pub struct Struct {
    x: u32,
    y: b256,
    z: b256,
}

pub enum Enum {
    V1: b256,
    V2: u64,
    V3: b256,
}

storage {
    map1: StorageMap<u64,
    bool>, map2: StorageMap<u64,
    u8>, map3: StorageMap<u64,
    u16>, map4: StorageMap<u64,
    u32>, map5: StorageMap<u64,
    u64>, map6: StorageMap<u64,
    (b256, u8, bool) >, map7: StorageMap<u64,
    Struct>, map8: StorageMap<u64,
    Enum>, map9: StorageMap<u64,
    str[33]>, map10: StorageMap<u64,
    [b256;
    3]>, map11: StorageMap<bool,
    u64>, map12: StorageMap<u8,
    u64>, map13: StorageMap<u16,
    u64>, map14: StorageMap<u32,
    u64>, map15: StorageMap<(b256,
    u8, bool), u64 >, map16: StorageMap<Struct,
    u64>, map17: StorageMap<Enum,
    u64>, map18: StorageMap<str[33],
    u64>, map19: StorageMap<[b256;
    3],
    u64>, 
}

abi StorageMapTest {
    fn init();

    fn insert_into_u64_to_bool_map(key: u64, value: bool);
    fn get_from_u64_to_bool_map(key: u64) -> bool;

    fn insert_into_u64_to_u8_map(key: u64, value: u8);
    fn get_from_u64_to_u8_map(key: u64) -> u8;

    fn insert_into_u64_to_u16_map(key: u64, value: u16);
    fn get_from_u64_to_u16_map(key: u64) -> u16;

    fn insert_into_u64_to_u32_map(key: u64, value: u32);
    fn get_from_u64_to_u32_map(key: u64) -> u32;

    fn insert_into_u64_to_u64_map(key: u64, value: u64);
    fn get_from_u64_to_u64_map(key: u64) -> u64;

    fn insert_into_u64_to_tuple_map(key: u64, value: (b256, u8, bool));
    fn get_from_u64_to_tuple_map(key: u64) -> (b256, u8, bool);

    fn insert_into_u64_to_struct_map(key: u64, value: Struct);
    fn get_from_u64_to_struct_map(key: u64) -> Struct;

    fn insert_into_u64_to_enum_map(key: u64, value: Enum);
    fn get_from_u64_to_enum_map(key: u64) -> Enum;

    fn insert_into_u64_to_str_map(key: u64, value: str[33]);
    fn get_from_u64_to_str_map(key: u64) -> str[33];

    fn insert_into_u64_to_array_map(key: u64, value: [b256;
    3]);
    fn get_from_u64_to_array_map(key: u64) -> [b256;
    3];

    fn insert_into_bool_to_u64_map(key: bool, value: u64);
    fn get_from_bool_to_u64_map(key: bool) -> u64;

    fn insert_into_u8_to_u64_map(key: u8, value: u64);
    fn get_from_u8_to_u64_map(key: u8) -> u64;

    fn insert_into_u16_to_u64_map(key: u16, value: u64);
    fn get_from_u16_to_u64_map(key: u16) -> u64;

    fn insert_into_u32_to_u64_map(key: u32, value: u64);
    fn get_from_u32_to_u64_map(key: u32) -> u64;

    fn insert_into_tuple_to_u64_map(key: (b256, u8, bool), value: u64);
    fn get_from_tuple_to_u64_map(key: (b256, u8, bool)) -> u64;

    fn insert_into_struct_to_u64_map(key: Struct, value: u64);
    fn get_from_struct_to_u64_map(key: Struct) -> u64;

    fn insert_into_enum_to_u64_map(key: Enum, value: u64);
    fn get_from_enum_to_u64_map(key: Enum) -> u64;

    fn insert_into_str_to_u64_map(key: str[33], value: u64);
    fn get_from_str_to_u64_map(key: str[33]) -> u64;

    fn insert_into_array_to_u64_map(key: [b256;
    3], value: u64);
    fn get_from_array_to_u64_map(key: [b256;
    3]) -> u64;
}

impl StorageMapTest for Contract {
    fn init() {
        storage.map1 = ~StorageMap::new::<u64, bool>();
        storage.map2 = ~StorageMap::new::<u64, u8>();
        storage.map3 = ~StorageMap::new::<u64, u16>();
        storage.map4 = ~StorageMap::new::<u64, u32>();
        storage.map5 = ~StorageMap::new::<u64, u64>();
        storage.map6 = ~StorageMap::new::<u64, (b256, u8, bool) >();
        storage.map7 = ~StorageMap::new::<u64, Struct>();
        storage.map8 = ~StorageMap::new::<u64, Enum>();
        storage.map9 = ~StorageMap::new::<u64, str[33]>();
        storage.map10 = ~StorageMap::new::<u64, [b256;
        3]>();

        storage.map11 = ~StorageMap::new::<bool, u64>();
        storage.map12 = ~StorageMap::new::<u8, u64>();
        storage.map13 = ~StorageMap::new::<u16, u64>();
        storage.map14 = ~StorageMap::new::<u32, u64>();
        storage.map15 = ~StorageMap::new::<(b256, u8, bool), u64 >();
        storage.map16 = ~StorageMap::new::<Struct, u64>();
        storage.map17 = ~StorageMap::new::<Enum, u64>();
        storage.map18 = ~StorageMap::new::<str[33], u64>();
        storage.map19 = ~StorageMap::new::<[b256;
        3], u64>();
    }

    fn insert_into_u64_to_bool_map(key: u64, value: bool) {
        storage.map1.insert(key, value);
    }

    fn get_from_u64_to_bool_map(key: u64) -> bool {
        storage.map1.get(key)
    }

    fn insert_into_u64_to_u8_map(key: u64, value: u8) {
        storage.map2.insert(key, value);
    }

    fn get_from_u64_to_u8_map(key: u64) -> u8 {
        storage.map2.get(key)
    }

    fn insert_into_u64_to_u16_map(key: u64, value: u16) {
        storage.map3.insert(key, value);
    }

    fn get_from_u64_to_u16_map(key: u64) -> u16 {
        storage.map3.get(key)
    }

    fn insert_into_u64_to_u32_map(key: u64, value: u32) {
        storage.map4.insert(key, value);
    }

    fn get_from_u64_to_u32_map(key: u64) -> u32 {
        storage.map4.get(key)
    }

    fn insert_into_u64_to_u64_map(key: u64, value: u64) {
        storage.map5.insert(key, value);
    }

    fn get_from_u64_to_u64_map(key: u64) -> u64 {
        storage.map5.get(key)
    }

    fn insert_into_u64_to_tuple_map(key: u64, value: (b256, u8, bool)) {
        storage.map6.insert(key, value);
    }

    fn get_from_u64_to_tuple_map(key: u64) -> (b256, u8, bool) {
        storage.map6.get(key)
    }

    fn insert_into_u64_to_struct_map(key: u64, value: Struct) {
        storage.map7.insert(key, value);
    }

    fn get_from_u64_to_struct_map(key: u64) -> Struct {
        storage.map7.get(key)
    }

    fn insert_into_u64_to_enum_map(key: u64, value: Enum) {
        storage.map8.insert(key, value);
    }

    fn get_from_u64_to_enum_map(key: u64) -> Enum {
        storage.map8.get(key)
    }

    fn insert_into_u64_to_str_map(key: u64, value: str[33]) {
        storage.map9.insert(key, value);
    }

    fn get_from_u64_to_str_map(key: u64) -> str[33] {
        storage.map9.get(key)
    }

    fn insert_into_u64_to_array_map(key: u64, value: [b256;
    3]) {
        storage.map10.insert(key, value);
    }

    fn get_from_u64_to_array_map(key: u64) -> [b256;
    3] {
        storage.map10.get(key)
    }

    fn insert_into_bool_to_u64_map(key: bool, value: u64) {
        storage.map11.insert(key, value);
    }
    fn get_from_bool_to_u64_map(key: bool) -> u64 {
        storage.map11.get(key)
    }

    fn insert_into_u8_to_u64_map(key: u8, value: u64) {
        storage.map12.insert(key, value);
    }
    fn get_from_u8_to_u64_map(key: u8) -> u64 {
        storage.map12.get(key)
    }

    fn insert_into_u16_to_u64_map(key: u16, value: u64) {
        storage.map13.insert(key, value);
    }
    fn get_from_u16_to_u64_map(key: u16) -> u64 {
        storage.map13.get(key)
    }

    fn insert_into_u32_to_u64_map(key: u32, value: u64) {
        storage.map14.insert(key, value);
    }
    fn get_from_u32_to_u64_map(key: u32) -> u64 {
        storage.map14.get(key)
    }

    fn insert_into_tuple_to_u64_map(key: (b256, u8, bool), value: u64) {
        storage.map15.insert(key, value);
    }
    fn get_from_tuple_to_u64_map(key: (b256, u8, bool)) -> u64 {
        storage.map15.get(key)
    }

    fn insert_into_struct_to_u64_map(key: Struct, value: u64) {
        storage.map16.insert(key, value);
    }

    fn get_from_struct_to_u64_map(key: Struct) -> u64 {
        storage.map16.get(key)
    }

    fn insert_into_enum_to_u64_map(key: Enum, value: u64) {
        storage.map17.insert(key, value);
    }

    fn get_from_enum_to_u64_map(key: Enum) -> u64 {
        storage.map17.get(key)
    }

    fn insert_into_str_to_u64_map(key: str[33], value: u64) {
        storage.map18.insert(key, value);
    }
    fn get_from_str_to_u64_map(key: str[33]) -> u64 {
        storage.map18.get(key)
    }

    fn insert_into_array_to_u64_map(key: [b256;
    3], value: u64) {
        storage.map19.insert(key, value)
    }
    fn get_from_array_to_u64_map(key: [b256;
    3]) -> u64 {
        storage.map19.get(key)
    }
}
