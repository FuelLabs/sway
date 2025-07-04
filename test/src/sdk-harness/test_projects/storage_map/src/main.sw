contract;

use std::hash::*;
use std::storage::storage_map::StorageMapError;

pub struct Struct {
    x: u32,
    y: b256,
    z: b256,
}

impl Hash for Struct {
    fn hash(self, ref mut state: Hasher) {
        self.x.hash(state);
        self.y.hash(state);
        self.z.hash(state);
    }
}

pub enum Enum {
    V1: b256,
    V2: u64,
    V3: b256,
}

impl Hash for Enum {
    fn hash(self, ref mut state: Hasher) {
        match self {
            Enum::V1(val) => {
                0_u8.hash(state);
                val.hash(state);
            }
            Enum::V2(val) => {
                1_u8.hash(state);
                val.hash(state);
            }
            Enum::V3(val) => {
                2_u8.hash(state);
                val.hash(state);
            }
        }
    }
}

storage {
    map1: StorageMap<u64, bool> = StorageMap::<u64, bool> {},
    map2: StorageMap<u64, u8> = StorageMap::<u64, u8> {},
    map3: StorageMap<u64, u16> = StorageMap::<u64, u16> {},
    map4: StorageMap<u64, u32> = StorageMap::<u64, u32> {},
    map5: StorageMap<u64, u64> = StorageMap::<u64, u64> {},
    map6: StorageMap<u64, (b256, u8, bool)> = StorageMap::<u64, (b256, u8, bool)> {},
    map7: StorageMap<u64, Struct> = StorageMap::<u64, Struct> {},
    map8: StorageMap<u64, Enum> = StorageMap::<u64, Enum> {},
    map9: StorageMap<u64, str[10]> = StorageMap::<u64, str[10]> {},
    map10: StorageMap<u64, [b256; 3]> = StorageMap::<u64, [b256; 3]> {},
    map11: StorageMap<bool, u64> = StorageMap::<bool, u64> {},
    map12: StorageMap<u8, u64> = StorageMap::<u8, u64> {},
    map13: StorageMap<u16, u64> = StorageMap::<u16, u64> {},
    map14: StorageMap<u32, u64> = StorageMap::<u32, u64> {},
    map15: StorageMap<(b256, u8, bool), u64> = StorageMap::<(b256, u8, bool), u64> {},
    map16: StorageMap<Struct, u64> = StorageMap::<Struct, u64> {},
    map17: StorageMap<Enum, u64> = StorageMap::<Enum, u64> {},
    map18: StorageMap<str[10], u64> = StorageMap::<str[10], u64> {},
    map19: StorageMap<[b256; 3], u64> = StorageMap::<[b256; 3], u64> {},
}

abi StorageMapTest {
    #[storage(read, write)]
    fn insert_into_u64_to_bool_map(key: u64, value: bool);
    #[storage(read)]
    fn get_from_u64_to_bool_map(key: u64) -> Option<bool>;
    #[storage(write)]
    fn remove_from_u64_to_bool_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_bool_map(key: u64, value: bool) -> Result<bool, StorageMapError<bool>>;

    #[storage(read, write)]
    fn insert_into_u64_to_u8_map(key: u64, value: u8);
    #[storage(read)]
    fn get_from_u64_to_u8_map(key: u64) -> Option<u8>;
    #[storage(write)]
    fn remove_from_u64_to_u8_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_u8_map(key: u64, value: u8) -> Result<u8, StorageMapError<u8>>;

    #[storage(read, write)]
    fn insert_into_u64_to_u16_map(key: u64, value: u16);
    #[storage(read)]
    fn get_from_u64_to_u16_map(key: u64) -> Option<u16>;
    #[storage(write)]
    fn remove_from_u64_to_u16_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_u16_map(key: u64, value: u16) -> Result<u16, StorageMapError<u16>>;

    #[storage(read, write)]
    fn insert_into_u64_to_u32_map(key: u64, value: u32);
    #[storage(read)]
    fn get_from_u64_to_u32_map(key: u64) -> Option<u32>;
    #[storage(write)]
    fn remove_from_u64_to_u32_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_u32_map(key: u64, value: u32) -> Result<u32, StorageMapError<u32>>;

    #[storage(read, write)]
    fn insert_into_u64_to_u64_map(key: u64, value: u64);
    #[storage(read)]
    fn get_from_u64_to_u64_map(key: u64) -> Option<u64>;
    #[storage(write)]
    fn remove_from_u64_to_u64_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_u64_map(key: u64, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_u64_to_tuple_map(key: u64, value: (b256, u8, bool));
    #[storage(read)]
    fn get_from_u64_to_tuple_map(key: u64) -> Option<(b256, u8, bool)>;
    #[storage(write)]
    fn remove_from_u64_to_tuple_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_tuple_map(
        key: u64,
        value: (b256, u8, bool),
    ) -> Result<(b256, u8, bool), StorageMapError<(b256, u8, bool)>>;

    #[storage(read, write)]
    fn insert_into_u64_to_struct_map(key: u64, value: Struct);
    #[storage(read)]
    fn get_from_u64_to_struct_map(key: u64) -> Option<Struct>;
    #[storage(write)]
    fn remove_from_u64_to_struct_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_struct_map(
        key: u64,
        value: Struct,
    ) -> Result<Struct, StorageMapError<Struct>>;

    #[storage(read, write)]
    fn insert_into_u64_to_enum_map(key: u64, value: Enum);
    #[storage(read)]
    fn get_from_u64_to_enum_map(key: u64) -> Option<Enum>;
    #[storage(write)]
    fn remove_from_u64_to_enum_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_enum_map(key: u64, value: Enum) -> Result<Enum, StorageMapError<Enum>>;

    #[storage(read, write)]
    fn insert_into_u64_to_str_map(key: u64, value: str[10]);
    #[storage(read)]
    fn get_from_u64_to_str_map(key: u64) -> Option<str[10]>;
    #[storage(write)]
    fn remove_from_u64_to_str_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_str_map(
        key: u64,
        value: str[10],
    ) -> Result<str[10], StorageMapError<str[10]>>;

    #[storage(read, write)]
    fn insert_into_u64_to_array_map(key: u64, value: [b256; 3]);
    #[storage(read)]
    fn get_from_u64_to_array_map(key: u64) -> Option<[b256; 3]>;
    #[storage(write)]
    fn remove_from_u64_to_array_map(key: u64) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u64_to_array_map(
        key: u64,
        value: [b256; 3],
    ) -> Result<[b256; 3], StorageMapError<[b256; 3]>>;

    #[storage(read, write)]
    fn insert_into_bool_to_u64_map(key: bool, value: u64);
    #[storage(read)]
    fn get_from_bool_to_u64_map(key: bool) -> Option<u64>;
    #[storage(write)]
    fn remove_from_bool_to_u64_map(key: bool) -> bool;
    #[storage(read, write)]
    fn try_insert_into_bool_to_u64_map(key: bool, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_u8_to_u64_map(key: u8, value: u64);
    #[storage(read)]
    fn get_from_u8_to_u64_map(key: u8) -> Option<u64>;
    #[storage(write)]
    fn remove_from_u8_to_u64_map(key: u8) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u8_to_u64_map(key: u8, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_u16_to_u64_map(key: u16, value: u64);
    #[storage(read)]
    fn get_from_u16_to_u64_map(key: u16) -> Option<u64>;
    #[storage(write)]
    fn remove_from_u16_to_u64_map(key: u16) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u16_to_u64_map(key: u16, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_u32_to_u64_map(key: u32, value: u64);
    #[storage(read)]
    fn get_from_u32_to_u64_map(key: u32) -> Option<u64>;
    #[storage(write)]
    fn remove_from_u32_to_u64_map(key: u32) -> bool;
    #[storage(read, write)]
    fn try_insert_into_u32_to_u64_map(key: u32, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_tuple_to_u64_map(key: (b256, u8, bool), value: u64);
    #[storage(read)]
    fn get_from_tuple_to_u64_map(key: (b256, u8, bool)) -> Option<u64>;
    #[storage(write)]
    fn remove_from_tuple_to_u64_map(key: (b256, u8, bool)) -> bool;
    #[storage(read, write)]
    fn try_insert_into_tuple_to_u64_map(
        key: (b256, u8, bool),
        value: u64,
    ) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_struct_to_u64_map(key: Struct, value: u64);
    #[storage(read)]
    fn get_from_struct_to_u64_map(key: Struct) -> Option<u64>;
    #[storage(write)]
    fn remove_from_struct_to_u64_map(key: Struct) -> bool;
    #[storage(read, write)]
    fn try_insert_into_struct_to_u64_map(key: Struct, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_enum_to_u64_map(key: Enum, value: u64);
    #[storage(read)]
    fn get_from_enum_to_u64_map(key: Enum) -> Option<u64>;
    #[storage(write)]
    fn remove_from_enum_to_u64_map(key: Enum) -> bool;
    #[storage(read, write)]
    fn try_insert_into_enum_to_u64_map(key: Enum, value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_str_to_u64_map(key: str[10], value: u64);
    #[storage(read)]
    fn get_from_str_to_u64_map(key: str[10]) -> Option<u64>;
    #[storage(write)]
    fn remove_from_str_to_u64_map(key: str[10]) -> bool;
    #[storage(read, write)]
    fn try_insert_into_str_to_u64_map(key: str[10], value: u64) -> Result<u64, StorageMapError<u64>>;

    #[storage(read, write)]
    fn insert_into_array_to_u64_map(key: [b256; 3], value: u64);
    #[storage(read)]
    fn get_from_array_to_u64_map(key: [b256; 3]) -> Option<u64>;
    #[storage(write)]
    fn remove_from_array_to_u64_map(key: [b256; 3]) -> bool;
    #[storage(read, write)]
    fn try_insert_into_array_to_u64_map(
        key: [b256; 3],
        value: u64,
    ) -> Result<u64, StorageMapError<u64>>;
}

impl StorageMapTest for Contract {
    #[storage(read, write)]
    fn insert_into_u64_to_bool_map(key: u64, value: bool) {
        storage.map1.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_bool_map(key: u64) -> Option<bool> {
        storage.map1.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_bool_map(key: u64) -> bool {
        storage.map1.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_bool_map(key: u64, value: bool) -> Result<bool, StorageMapError<bool>> {
        storage.map1.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_u8_map(key: u64, value: u8) {
        storage.map2.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u8_map(key: u64) -> Option<u8> {
        storage.map2.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_u8_map(key: u64) -> bool {
        storage.map2.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_u8_map(key: u64, value: u8) -> Result<u8, StorageMapError<u8>> {
        storage.map2.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_u16_map(key: u64, value: u16) {
        storage.map3.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u16_map(key: u64) -> Option<u16> {
        storage.map3.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_u16_map(key: u64) -> bool {
        storage.map3.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_u16_map(key: u64, value: u16) -> Result<u16, StorageMapError<u16>> {
        storage.map3.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_u32_map(key: u64, value: u32) {
        storage.map4.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u32_map(key: u64) -> Option<u32> {
        storage.map4.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_u32_map(key: u64) -> bool {
        storage.map4.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_u32_map(key: u64, value: u32) -> Result<u32, StorageMapError<u32>> {
        storage.map4.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_u64_map(key: u64, value: u64) {
        storage.map5.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_u64_map(key: u64) -> Option<u64> {
        storage.map5.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_u64_map(key: u64) -> bool {
        storage.map5.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_u64_map(key: u64, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map5.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_tuple_map(key: u64, value: (b256, u8, bool)) {
        storage.map6.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_tuple_map(key: u64) -> Option<(b256, u8, bool)> {
        storage.map6.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_tuple_map(key: u64) -> bool {
        storage.map6.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_tuple_map(
        key: u64,
        value: (b256, u8, bool),
    ) -> Result<(b256, u8, bool), StorageMapError<(b256, u8, bool)>> {
        storage.map6.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_struct_map(key: u64, value: Struct) {
        storage.map7.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_struct_map(key: u64) -> Option<Struct> {
        storage.map7.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_struct_map(key: u64) -> bool {
        storage.map7.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_struct_map(
        key: u64,
        value: Struct,
    ) -> Result<Struct, StorageMapError<Struct>> {
        storage.map7.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_enum_map(key: u64, value: Enum) {
        storage.map8.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_enum_map(key: u64) -> Option<Enum> {
        storage.map8.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_enum_map(key: u64) -> bool {
        storage.map8.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_enum_map(key: u64, value: Enum) -> Result<Enum, StorageMapError<Enum>> {
        storage.map8.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_str_map(key: u64, value: str[10]) {
        storage.map9.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_str_map(key: u64) -> Option<str[10]> {
        storage.map9.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_str_map(key: u64) -> bool {
        storage.map9.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_str_map(
        key: u64,
        value: str[10],
    ) -> Result<str[10], StorageMapError<str[10]>> {
        storage.map9.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u64_to_array_map(key: u64, value: [b256; 3]) {
        storage.map10.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u64_to_array_map(key: u64) -> Option<[b256; 3]> {
        storage.map10.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u64_to_array_map(key: u64) -> bool {
        storage.map10.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u64_to_array_map(
        key: u64,
        value: [b256; 3],
    ) -> Result<[b256; 3], StorageMapError<[b256; 3]>> {
        storage.map10.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_bool_to_u64_map(key: bool, value: u64) {
        storage.map11.insert(key, value);
    }

    #[storage(read)]
    fn get_from_bool_to_u64_map(key: bool) -> Option<u64> {
        storage.map11.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_bool_to_u64_map(key: bool) -> bool {
        storage.map11.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_bool_to_u64_map(key: bool, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map11.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u8_to_u64_map(key: u8, value: u64) {
        storage.map12.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u8_to_u64_map(key: u8) -> Option<u64> {
        storage.map12.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u8_to_u64_map(key: u8) -> bool {
        storage.map12.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u8_to_u64_map(key: u8, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map12.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u16_to_u64_map(key: u16, value: u64) {
        storage.map13.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u16_to_u64_map(key: u16) -> Option<u64> {
        storage.map13.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u16_to_u64_map(key: u16) -> bool {
        storage.map13.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u16_to_u64_map(key: u16, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map13.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_u32_to_u64_map(key: u32, value: u64) {
        storage.map14.insert(key, value);
    }

    #[storage(read)]
    fn get_from_u32_to_u64_map(key: u32) -> Option<u64> {
        storage.map14.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_u32_to_u64_map(key: u32) -> bool {
        storage.map14.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_u32_to_u64_map(key: u32, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map14.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_tuple_to_u64_map(key: (b256, u8, bool), value: u64) {
        storage.map15.insert(key, value);
    }

    #[storage(read)]
    fn get_from_tuple_to_u64_map(key: (b256, u8, bool)) -> Option<u64> {
        storage.map15.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_tuple_to_u64_map(key: (b256, u8, bool)) -> bool {
        storage.map15.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_tuple_to_u64_map(
        key: (b256, u8, bool),
        value: u64,
    ) -> Result<u64, StorageMapError<u64>> {
        storage.map15.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_struct_to_u64_map(key: Struct, value: u64) {
        storage.map16.insert(key, value);
    }

    #[storage(read)]
    fn get_from_struct_to_u64_map(key: Struct) -> Option<u64> {
        storage.map16.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_struct_to_u64_map(key: Struct) -> bool {
        storage.map16.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_struct_to_u64_map(key: Struct, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map16.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_enum_to_u64_map(key: Enum, value: u64) {
        storage.map17.insert(key, value);
    }

    #[storage(read)]
    fn get_from_enum_to_u64_map(key: Enum) -> Option<u64> {
        storage.map17.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_enum_to_u64_map(key: Enum) -> bool {
        storage.map17.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_enum_to_u64_map(key: Enum, value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map17.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_str_to_u64_map(key: str[10], value: u64) {
        storage.map18.insert(key, value);
    }

    #[storage(read)]
    fn get_from_str_to_u64_map(key: str[10]) -> Option<u64> {
        storage.map18.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_str_to_u64_map(key: str[10]) -> bool {
        storage.map18.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_str_to_u64_map(key: str[10], value: u64) -> Result<u64, StorageMapError<u64>> {
        storage.map18.try_insert(key, value)
    }

    #[storage(read, write)]
    fn insert_into_array_to_u64_map(key: [b256; 3], value: u64) {
        storage.map19.insert(key, value)
    }

    #[storage(read)]
    fn get_from_array_to_u64_map(key: [b256; 3]) -> Option<u64> {
        storage.map19.get(key).try_read()
    }

    #[storage(write)]
    fn remove_from_array_to_u64_map(key: [b256; 3]) -> bool {
        storage.map19.remove(key)
    }

    #[storage(read, write)]
    fn try_insert_into_array_to_u64_map(
        key: [b256; 3],
        value: u64,
    ) -> Result<u64, StorageMapError<u64>> {
        storage.map19.try_insert(key, value)
    }
}
