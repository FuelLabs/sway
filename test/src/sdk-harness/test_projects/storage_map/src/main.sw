contract;

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
    map1: StorageMap<u64, bool> = StorageMap {},
    map2: StorageMap<u64, u8> = StorageMap {},
    map3: StorageMap<u64, u16> = StorageMap {},
    map4: StorageMap<u64, u32> = StorageMap {},
    map5: StorageMap<u64, u64> = StorageMap {},
    map6: StorageMap<u64, (b256, u8, bool)> = StorageMap {},
    map7: StorageMap<u64, Struct> = StorageMap {},
    map8: StorageMap<u64, Enum> = StorageMap {},
    map9: StorageMap<u64, str[33]> = StorageMap {},
    map10: StorageMap<u64, [b256; 3]> = StorageMap {},
    map11: StorageMap<bool, u64> = StorageMap {},
    map12: StorageMap<u8, u64> = StorageMap {},
    map13: StorageMap<u16, u64> = StorageMap {},
    map14: StorageMap<u32, u64> = StorageMap {},
    map15: StorageMap<(b256, u8, bool), u64> = StorageMap {},
    map16: StorageMap<Struct, u64> = StorageMap {},
    map17: StorageMap<Enum, u64> = StorageMap {},
    map18: StorageMap<str[33], u64> = StorageMap {},
    map19: StorageMap<[b256; 3], u64> = StorageMap {},
}

abi StorageMapTest {
    #[storage(write)]
    fn insert_into_u64_to_bool_map(key: u64, value: bool);
    #[storage(read)]
    fn get_from_u64_to_bool_map(key: u64) -> bool;

    #[storage(write)]
    fn insert_into_u64_to_u8_map(key: u64, value: u8);
    #[storage(read)]
    fn get_from_u64_to_u8_map(key: u64) -> u8;

    #[storage(write)]
    fn insert_into_u64_to_u16_map(key: u64, value: u16);
    #[storage(read)]
    fn get_from_u64_to_u16_map(key: u64) -> u16;

    #[storage(write)]
    fn insert_into_u64_to_u32_map(key: u64, value: u32);
    #[storage(read)]
    fn get_from_u64_to_u32_map(key: u64) -> u32;

    #[storage(write)]
    fn insert_into_u64_to_u64_map(key: u64, value: u64);
    #[storage(read)]
    fn get_from_u64_to_u64_map(key: u64) -> u64;

    #[storage(write)]
    fn insert_into_u64_to_tuple_map(key: u64, value: (b256, u8, bool));
    #[storage(read)]
    fn get_from_u64_to_tuple_map(key: u64) -> (b256, u8, bool);

    #[storage(write)]
    fn insert_into_u64_to_struct_map(key: u64, value: Struct);
    #[storage(read)]
    fn get_from_u64_to_struct_map(key: u64) -> Struct;

    #[storage(write)]
    fn insert_into_u64_to_enum_map(key: u64, value: Enum);
    #[storage(read)]
    fn get_from_u64_to_enum_map(key: u64) -> Enum;

    #[storage(write)]
    fn insert_into_u64_to_str_map(key: u64, value: str[33]);
    #[storage(read)]
    fn get_from_u64_to_str_map(key: u64) -> str[33];

    #[storage(write)]
    fn insert_into_u64_to_array_map(key: u64, value: [b256; 3]);
    #[storage(read)]
    fn get_from_u64_to_array_map(key: u64) -> [b256; 3];

    #[storage(write)]
    fn insert_into_bool_to_u64_map(key: bool, value: u64);
    #[storage(read)]
    fn get_from_bool_to_u64_map(key: bool) -> u64;

    #[storage(write)]
    fn insert_into_u8_to_u64_map(key: u8, value: u64);
    #[storage(read)]
    fn get_from_u8_to_u64_map(key: u8) -> u64;

    #[storage(write)]
    fn insert_into_u16_to_u64_map(key: u16, value: u64);
    #[storage(read)]
    fn get_from_u16_to_u64_map(key: u16) -> u64;

    #[storage(write)]
    fn insert_into_u32_to_u64_map(key: u32, value: u64);
    #[storage(read)]
    fn get_from_u32_to_u64_map(key: u32) -> u64;

    #[storage(write)]
    fn insert_into_tuple_to_u64_map(key: (b256, u8, bool), value: u64);
    #[storage(read)]
    fn get_from_tuple_to_u64_map(key: (b256, u8, bool)) -> u64;

    #[storage(write)]
    fn insert_into_struct_to_u64_map(key: Struct, value: u64);
    #[storage(read)]
    fn get_from_struct_to_u64_map(key: Struct) -> u64;

    #[storage(write)]
    fn insert_into_enum_to_u64_map(key: Enum, value: u64);
    #[storage(read)]
    fn get_from_enum_to_u64_map(key: Enum) -> u64;

    #[storage(write)]
    fn insert_into_str_to_u64_map(key: str[33], value: u64);
    #[storage(read)]
    fn get_from_str_to_u64_map(key: str[33]) -> u64;

    #[storage(write)]
    fn insert_into_array_to_u64_map(key: [b256; 3], value: u64);
    #[storage(read)]
    fn get_from_array_to_u64_map(key: [b256; 3]) -> u64;
}

#[storage(write)]
fn _insert_into_u64_to_bool_map_inner(key: u64, value: bool) {
    storage.map1.insert(key, value);
}

#[storage(read)]
fn _get_from_u64_to_bool_map_inner(key: u64) -> bool {
    storage.map1.get(key)
}

impl StorageMapTest for Contract {
    #[storage(write)]
    fn insert_into_u64_to_bool_map(key: u64, value: bool) {
        _insert_into_u64_to_bool_map_inner(key, value)
    }

    #[storage(read)]
    fn get_from_u64_to_bool_map(key: u64) -> bool {
        _get_from_u64_to_bool_map_inner(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_u8_map(key: u64, value: u8) {
        storage.map2.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u8_map(key: u64) -> u8 {
        storage.map2.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_u16_map(key: u64, value: u16) {
        storage.map3.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u16_map(key: u64) -> u16 {
        storage.map3.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_u32_map(key: u64, value: u32) {
        storage.map4.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u32_map(key: u64) -> u32 {
        storage.map4.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_u64_map(key: u64, value: u64) {
        storage.map5.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u64_map(key: u64) -> u64 {
        storage.map5.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_tuple_map(key: u64, value: (b256, u8, bool)) {
        storage.map6.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_tuple_map(key: u64) -> (b256, u8, bool) {
        storage.map6.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_struct_map(key: u64, value: Struct) {
        storage.map7.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_struct_map(key: u64) -> Struct {
        storage.map7.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_enum_map(key: u64, value: Enum) {
        storage.map8.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_enum_map(key: u64) -> Enum {
        storage.map8.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_str_map(key: u64, value: str[33]) {
        storage.map9.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_str_map(key: u64) -> str[33] {
        storage.map9.get(key)
    }

    #[storage(write)]
    fn insert_into_u64_to_array_map(key: u64, value: [b256; 3]) {
        storage.map10.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_array_map(key: u64) -> [b256; 3] {
        storage.map10.get(key)
    }

    #[storage(write)]
    fn insert_into_bool_to_u64_map(key: bool, value: u64) {
        storage.map11.insert(key, value);
    }
    #[storage(read)]
    fn get_from_bool_to_u64_map(key: bool) -> u64 {
        storage.map11.get(key)
    }

    #[storage(write)]
    fn insert_into_u8_to_u64_map(key: u8, value: u64) {
        storage.map12.insert(key, value);
    }
    #[storage(read)]
    fn get_from_u8_to_u64_map(key: u8) -> u64 {
        storage.map12.get(key)
    }

    #[storage(write)]
    fn insert_into_u16_to_u64_map(key: u16, value: u64) {
        storage.map13.insert(key, value);
    }
    #[storage(read)]
    fn get_from_u16_to_u64_map(key: u16) -> u64 {
        storage.map13.get(key)
    }

    #[storage(write)]
    fn insert_into_u32_to_u64_map(key: u32, value: u64) {
        storage.map14.insert(key, value);
    }
    #[storage(read)]
    fn get_from_u32_to_u64_map(key: u32) -> u64 {
        storage.map14.get(key)
    }

    #[storage(write)]
    fn insert_into_tuple_to_u64_map(key: (b256, u8, bool), value: u64) {
        storage.map15.insert(key, value);
    }
    #[storage(read)]
    fn get_from_tuple_to_u64_map(key: (b256, u8, bool)) -> u64 {
        storage.map15.get(key)
    }

    #[storage(write)]
    fn insert_into_struct_to_u64_map(key: Struct, value: u64) {
        storage.map16.insert(key, value);
    }

    #[storage(read)]
    fn get_from_struct_to_u64_map(key: Struct) -> u64 {
        storage.map16.get(key)
    }

    #[storage(write)]
    fn insert_into_enum_to_u64_map(key: Enum, value: u64) {
        storage.map17.insert(key, value);
    }

    #[storage(read)]
    fn get_from_enum_to_u64_map(key: Enum) -> u64 {
        storage.map17.get(key)
    }

    #[storage(write)]
    fn insert_into_str_to_u64_map(key: str[33], value: u64) {
        storage.map18.insert(key, value);
    }
    #[storage(read)]
    fn get_from_str_to_u64_map(key: str[33]) -> u64 {
        storage.map18.get(key)
    }

    #[storage(write)]
    fn insert_into_array_to_u64_map(key: [b256; 3], value: u64) {
        storage.map19.insert(key, value)
    }
    #[storage(read)]
    fn get_from_array_to_u64_map(key: [b256; 3]) -> u64 {
        storage.map19.get(key)
    }
}
