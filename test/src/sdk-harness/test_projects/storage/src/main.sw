contract;

use std::storage::{get, store};

pub enum MediumEnum {
    One: u64,
    Two: bool,
    Three: b256,
}

pub struct SmallStruct {
    x: u64,
}

pub struct MediumStruct {
    x: u64,
    y: u32,
}

pub struct LargeStruct {
    x: u32,
    y: b256,
    z: u32,
}

pub struct VeryLargeStruct {
    x: u32,
    y: b256,
    z: b256,
}

pub enum StorageEnum {
    V1: b256,
    V2: u64,
    V3: b256,
}

/// Storage delimiters
const S_1: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const S_2: b256 = 0x0000000000000000000000000000000000000000000000000000000000000002;
const S_3: b256 = 0x0000000000000000000000000000000000000000000000000000000000000003;
const S_4: b256 = 0x0000000000000000000000000000000000000000000000000000000000000004;
const S_5: b256 = 0x0000000000000000000000000000000000000000000000000000000000000005;
const S_6: b256 = 0x0000000000000000000000000000000000000000000000000000000000000006;
const S_7: b256 = 0x0000000000000000000000000000000000000000000000000000000000000007;
const S_8: b256 = 0x0000000000000000000000000000000000000000000000000000000000000008;
const S_9: b256 = 0x0000000000000000000000000000000000000000000000000000000000000009;
const S_10: b256 = 0x0000000000000000000000000000000000000000000000000000000000000010;
const S_11: b256 = 0x0000000000000000000000000000000000000000000000000000000000000011;
const S_12: b256 = 0x0000000000000000000000000000000000000000000000000000000000000012;
const S_13: b256 = 0x0000000000000000000000000000000000000000000000000000000000000013;
const S_14: b256 = 0x0000000000000000000000000000000000000000000000000000000000000014;

abi StorageTest {
    fn store_bool(value: bool);
    fn get_bool() -> bool;
    fn store_u8(value: u8);
    fn get_u8() -> u8;
    fn store_u16(value: u16);
    fn get_u16() -> u16;
    fn store_u32(value: u32);
    fn get_u32() -> u32;
    fn store_u64(value: u64);
    fn get_u64() -> u64;
    fn store_b256(value: b256);
    fn get_b256() -> b256;

    fn store_small_struct(value: SmallStruct);
    fn get_small_struct() -> SmallStruct;
    fn store_medium_struct(value: MediumStruct);
    fn get_medium_struct() -> MediumStruct;
    fn store_large_struct(value: LargeStruct);
    fn get_large_struct() -> LargeStruct;
    fn store_very_large_struct(value: VeryLargeStruct);
    fn get_very_large_struct() -> VeryLargeStruct;

    fn store_enum(value: StorageEnum);
    fn get_enum() -> StorageEnum;

    fn store_tuple(value: (b256, u8, b256));
    fn get_tuple() -> (b256, u8, b256);

    fn store_string(value: str[31]);
    fn get_string() -> str[31];

    fn store_array();
    fn get_array() -> [b256;
    3];
}

impl StorageTest for Contract {
    fn store_bool(value: bool) {
        store(S_1, value);
    }

    fn get_bool() -> bool {
        get(S_1)
    }

    fn store_u8(value: u8) {
        store(S_2, value);
    }

    fn get_u8() -> u8 {
        get(S_2)
    }

    fn store_u16(value: u16) {
        store(S_3, value);
    }

    fn get_u16() -> u16 {
        get(S_3)
    }

    fn store_u32(value: u32) {
        store(S_4, value);
    }

    fn get_u32() -> u32 {
        get(S_4)
    }

    fn store_u64(value: u64) {
        store(S_5, value);
    }

    fn get_u64() -> u64 {
        get(S_5)
    }

    fn store_b256(value: b256) {
        store(S_6, value);
    }

    fn get_b256() -> b256 {
        get(S_6)
    }

    fn store_small_struct(value: SmallStruct) {
        store(S_8, value);
    }

    fn get_small_struct() -> SmallStruct {
        get(S_8)
    }

    fn store_medium_struct(value: MediumStruct) {
        store(S_9, value);
    }

    fn get_medium_struct() -> MediumStruct {
        get(S_9)
    }

    fn store_large_struct(value: LargeStruct) {
        store(S_9, value);
    }

    fn get_large_struct() -> LargeStruct {
        get(S_9)
    }

    fn store_very_large_struct(value: VeryLargeStruct) {
        store(S_10, value);
    }

    fn get_very_large_struct() -> VeryLargeStruct {
        get(S_10)
    }

    fn store_enum(value: StorageEnum) {
        store(S_11, value);
    }

    fn get_enum() -> StorageEnum {
        get(S_11)
    }

    fn store_tuple(value: (b256, u8, b256)) {
        store(S_12, value);
    }

    fn get_tuple() -> (b256, u8, b256) {
        get(S_12)
    }

    fn store_string(value: str[31]) {
        store(S_13, value);
    }

    fn get_string() -> str[31] {
        get(S_13)
    }

    // Passing arrays into contract methods is not working at the moment
    fn store_array() {
        let a = [0x9999999999999999999999999999999999999999999999999999999999999999, 0x8888888888888888888888888888888888888888888888888888888888888888, 0x7777777777777777777777777777777777777777777777777777777777777777];
        store(S_14, a);
    }

    fn get_array() -> [b256;
    3] {
        get(S_14)
    }
}
