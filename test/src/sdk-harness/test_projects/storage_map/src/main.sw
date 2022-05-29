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

    fn into_u64_to_bool(key: u64, value: bool);
    fn from_u64_to_bool(key: u64) -> bool;

    fn into_u64_to_u8(key: u64, value: u8);
    fn from_u64_to_u8(key: u64) -> u8;

    fn into_u64_to_u16(key: u64, value: u16);
    fn from_u64_to_u16(key: u64) -> u16;

    fn into_u64_to_u32(key: u64, value: u32);
    fn from_u64_to_u32(key: u64) -> u32;

    fn into_u64_to_u64(key: u64, value: u64);
    fn from_u64_to_u64(key: u64) -> u64;

    fn into_u64_to_tuple(key: u64, value: (b256, u8, bool));
    fn from_u64_to_tuple(key: u64) -> (b256, u8, bool);

    fn into_u64_to_struct(key: u64, value: Struct);
    fn from_u64_to_struct(key: u64) -> Struct;

    fn into_u64_to_enum(key: u64, value: Enum);
    fn from_u64_to_enum(key: u64) -> Enum;

    fn into_u64_to_str(key: u64, value: str[33]);
    fn from_u64_to_str(key: u64) -> str[33];

    fn into_u64_to_array(key: u64, value: [b256;
    3]);
    fn from_u64_to_array(key: u64) -> [b256;
    3];

    fn into_bool_to_u64(key: bool, value: u64);
    fn from_bool_to_u64(key: bool) -> u64;

    fn into_u8_to_u64(key: u8, value: u64);
    fn from_u8_to_u64(key: u8) -> u64;

    fn into_u16_to_u64(key: u16, value: u64);
    fn from_u16_to_u64(key: u16) -> u64;

    fn into_u32_to_u64(key: u32, value: u64);
    fn from_u32_to_u64(key: u32) -> u64;

    fn into_tuple_to_u64(key: (b256, u8, bool), value: u64);
    fn from_tuple_to_u64(key: (b256, u8, bool)) -> u64;

    fn into_struct_to_u64(key: Struct, value: u64);
    fn from_struct_to_u64(key: Struct) -> u64;

    fn into_enum_to_u64(key: Enum, value: u64);
    fn from_enum_to_u64(key: Enum) -> u64;

    fn into_str_to_u64(key: str[33], value: u64);
    fn from_str_to_u64(key: str[33]) -> u64;

    fn into_array_to_u64(key: [b256;
    3], value: u64);
    fn from_array_to_u64(key: [b256;
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

    fn into_u64_to_bool(key: u64, value: bool) {
        storage.map1.insert(key, value);
    }

    fn from_u64_to_bool(key: u64) -> bool {
        storage.map1.get(key)
    }

    fn into_u64_to_u8(key: u64, value: u8) {
        storage.map2.insert(key, value);
    }

    fn from_u64_to_u8(key: u64) -> u8 {
        storage.map2.get(key)
    }

    fn into_u64_to_u16(key: u64, value: u16) {
        storage.map3.insert(key, value);
    }

    fn from_u64_to_u16(key: u64) -> u16 {
        storage.map3.get(key)
    }

    fn into_u64_to_u32(key: u64, value: u32) {
        storage.map4.insert(key, value);
    }

    fn from_u64_to_u32(key: u64) -> u32 {
        storage.map4.get(key)
    }

    fn into_u64_to_u64(key: u64, value: u64) {
        storage.map5.insert(key, value);
    }

    fn from_u64_to_u64(key: u64) -> u64 {
        storage.map5.get(key)
    }

    fn into_u64_to_tuple(key: u64, value: (b256, u8, bool)) {
        storage.map6.insert(key, value);
    }

    fn from_u64_to_tuple(key: u64) -> (b256, u8, bool) {
        storage.map6.get(key)
    }

    fn into_u64_to_struct(key: u64, value: Struct) {
        storage.map7.insert(key, value);
    }

    fn from_u64_to_struct(key: u64) -> Struct {
        storage.map7.get(key)
    }

    fn into_u64_to_enum(key: u64, value: Enum) {
        storage.map8.insert(key, value);
    }

    fn from_u64_to_enum(key: u64) -> Enum {
        storage.map8.get(key)
    }

    fn into_u64_to_str(key: u64, value: str[33]) {
        storage.map9.insert(key, value);
    }

    fn from_u64_to_str(key: u64) -> str[33] {
        storage.map9.get(key)
    }

    fn into_u64_to_array(key: u64, value: [b256;
    3]) {
        storage.map10.insert(key, value);
    }

    fn from_u64_to_array(key: u64) -> [b256;
    3] {
        storage.map10.get(key)
    }

    fn into_bool_to_u64(key: bool, value: u64) {
        storage.map11.insert(key, value);
    }
    fn from_bool_to_u64(key: bool) -> u64 {
        storage.map11.get(key)
    }

    fn into_u8_to_u64(key: u8, value: u64) {
        storage.map12.insert(key, value);
    }
    fn from_u8_to_u64(key: u8) -> u64 {
        storage.map12.get(key)
    }

    fn into_u16_to_u64(key: u16, value: u64) {
        storage.map13.insert(key, value);
    }
    fn from_u16_to_u64(key: u16) -> u64 {
        storage.map13.get(key)
    }

    fn into_u32_to_u64(key: u32, value: u64) {
        storage.map14.insert(key, value);
    }
    fn from_u32_to_u64(key: u32) -> u64 {
        storage.map14.get(key)
    }

    fn into_tuple_to_u64(key: (b256, u8, bool), value: u64) {
        storage.map15.insert(key, value);
    }
    fn from_tuple_to_u64(key: (b256, u8, bool)) -> u64 {
        storage.map15.get(key)
    }

    fn into_struct_to_u64(key: Struct, value: u64) {
        storage.map16.insert(key, value);
    }

    fn from_struct_to_u64(key: Struct) -> u64 {
        storage.map16.get(key)
    }

    fn into_enum_to_u64(key: Enum, value: u64) {
        storage.map17.insert(key, value);
    }

    fn from_enum_to_u64(key: Enum) -> u64 {
        storage.map17.get(key)
    }

    fn into_str_to_u64(key: str[33], value: u64) {
        storage.map18.insert(key, value);
    }
    fn from_str_to_u64(key: str[33]) -> u64 {
        storage.map18.get(key)
    }

    fn into_array_to_u64(key: [b256;
    3], value: u64) {
        storage.map19.insert(key, value)
    }
    fn from_array_to_u64(key: [b256;
    3]) -> u64 {
        storage.map19.get(key)
    }
}
