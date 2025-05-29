script;

use std::hash::*;

struct ConfigurableStruct {
    a: bool,
    b: u64,
}

enum ConfigurableEnum {
    A: bool,
    B: u64,
    C: b256,
}

impl PartialEq for ConfigurableEnum {
    fn eq(self, other: ConfigurableEnum) -> bool {
        match (self, other) {
            (ConfigurableEnum::A(inner1), ConfigurableEnum::A(inner2)) => inner1 == inner2,
            (ConfigurableEnum::B(inner1), ConfigurableEnum::B(inner2)) => inner1 == inner2,
            _ => false,
        }
    }
}
impl Eq for ConfigurableEnum {}

type AnotherU8 = u8;

configurable {
    BOOL: bool = true,
    U8: u8 = 1,
    ANOTHER_U8: AnotherU8 = 3,
    U16: u16 = 2,
    U32: u32 = 3,
    U64: u32 = 4,
    U256: u256 = 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAu256,
    B256: b256 = 0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB,
    CONFIGURABLE_STRUCT: ConfigurableStruct = ConfigurableStruct { a: true, b: 5 },
    CONFIGURABLE_ENUM_A: ConfigurableEnum = ConfigurableEnum::A(true),
    CONFIGURABLE_ENUM_B: ConfigurableEnum = ConfigurableEnum::B(12),
    ARRAY_BOOL: [bool; 3] = [true, false, true],
    ARRAY_U64: [u64; 3] = [9, 8, 7],
    TUPLE_BOOL_U64: (bool, u64) = (true, 11),
    STR_4: str[4] = __to_str_array("abcd"),
    NOT_USED: u8 = 1,
}

fn main() {
    assert(BOOL == true);
    assert(U8 == 1);
    assert(ANOTHER_U8 == 3);
    assert(U16 == 2);
    assert(U32 == 3);
    assert(U64 == 4);
    assert(U256 == 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAu256);
    assert(B256 == 0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB);
    assert(CONFIGURABLE_STRUCT.a == true);
    assert(CONFIGURABLE_STRUCT.b == 5);
    assert(CONFIGURABLE_ENUM_A == ConfigurableEnum::A(true));
    assert(CONFIGURABLE_ENUM_B == ConfigurableEnum::B(12));
    assert(ARRAY_BOOL[0] == true);
    assert(ARRAY_BOOL[1] == false);
    assert(ARRAY_BOOL[2] == true);
    assert(ARRAY_U64[0] == 9);
    assert(ARRAY_U64[1] == 8);
    assert(ARRAY_U64[2] == 7);
    assert(TUPLE_BOOL_U64.0 == true);
    assert(TUPLE_BOOL_U64.1 == 11);
    assert(sha256_str_array(STR_4) == sha256("abcd"));

    // Assert address do not change
    let addr_1 = asm(addr: &BOOL) {
        addr: u64
    };
    let addr_2 = asm(addr: &BOOL) {
        addr: u64
    };
    assert(addr_1 == addr_2);
}
