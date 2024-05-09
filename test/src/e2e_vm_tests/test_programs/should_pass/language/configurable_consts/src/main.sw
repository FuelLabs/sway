script;

use std::hash::*;

struct MyStruct {
    x: u64,
    y: bool,
}

enum MyEnum {
    A: u64,
    B: bool,
}

impl core::ops::Eq for MyEnum {
    fn eq(self, other: MyEnum) -> bool {
        match (self, other) {
            (MyEnum::A(inner1), MyEnum::A(inner2)) => inner1 == inner2,
            (MyEnum::B(inner1), MyEnum::B(inner2)) => inner1 == inner2,
            _ => false,
        }
    }
}

type AnotherU8 = u8;

configurable {
    C0: bool = true,
    C1: u64 = 42,
    C2: b256 = 0x1111111111111111111111111111111111111111111111111111111111111111,
    C3: MyStruct = MyStruct { x: 42, y: true },
    C4: MyEnum = MyEnum::A(42),
    C5: MyEnum = MyEnum::B(true),
    C6: str[4] = __to_str_array("fuel"),
    C7: [u64; 4] = [1, 2, 3, 4],
    
    C8: u64 = 0, // Unused - should not show up in the JSON file
    C9: u64 =  10 + 9 - 8 * 7 / 6 << 5 >> 4 ^ 3 | 2 & 1,

    UNIT: () = (),
    BOOL: bool = true,
    U8: u8 = 8,
    ANOTHERU8: AnotherU8 = 8,
    U16: u16 = 16,
    U32: u32 = 32,
    U64: u64 = 64,
    U256: u256 = 0x1234567812345678123456781234567812345678123456781234567812345678u256,
    ARRAY_U8: [u8; 4] = [1, 2, 3, 4],
    ARRAY_U32: [u32; 3] = [1, 2, 3],
}

#[inline(never)]
fn test_first_use() {
    assert(C0 == true);
    assert(C1 == 42);
    assert(C2 == 0x1111111111111111111111111111111111111111111111111111111111111111);
    assert(C3.x == 42);
    assert(C3.y == true);
    assert(C4 == MyEnum::A(42));
    assert(C5 == MyEnum::B(true));
    assert(sha256_str_array(C6) == sha256("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
    assert(C9 == 23);

    __log(UNIT);

    assert(BOOL);
    assert(U8 == 8);
    assert(ANOTHERU8 == 8);
    assert(U16 == 16);
    assert(U32 == 32);
    assert(U64 == 64);
    assert(U256 == 0x1234567812345678123456781234567812345678123456781234567812345678u256);

    assert(ARRAY_U8[0] == 1);
    assert(ARRAY_U8[1] == 2);
    assert(ARRAY_U8[2] == 3);
    assert(ARRAY_U8[3] == 4);
    
    assert(ARRAY_U32[0] == 1);
    assert(ARRAY_U32[1] == 2);
    assert(ARRAY_U32[2] == 3);
}

#[inline(never)]
fn test_second_use() {
    assert(C0 == true);
    assert(C1 == 42);
    assert(C2 == 0x1111111111111111111111111111111111111111111111111111111111111111);
    assert(C3.x == 42);
    assert(C3.y == true);
    assert(C4 == MyEnum::A(42));
    assert(C5 == MyEnum::B(true));
    assert(sha256_str_array(C6) == sha256("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
    assert(C9 == 23);

    assert(ARRAY_U8[0] == 1);
    assert(ARRAY_U8[1] == 2);
    assert(ARRAY_U8[2] == 3);
    assert(ARRAY_U8[3] == 4);

    assert(ARRAY_U32[0] == 1);
    assert(ARRAY_U32[1] == 2);
    assert(ARRAY_U32[2] == 3);
}

#[inline(always)]
fn test_inline_use() {
    assert(C0 == true);
    assert(C1 == 42);
    assert(C2 == 0x1111111111111111111111111111111111111111111111111111111111111111);
    assert(C3.x == 42);
    assert(C3.y == true);
    assert(C4 == MyEnum::A(42));
    assert(C5 == MyEnum::B(true));
    assert(sha256_str_array(C6) == sha256("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
    assert(C9 == 23);

    assert(ARRAY_U8[0] == 1);
    assert(ARRAY_U8[1] == 2);
    assert(ARRAY_U8[2] == 3);
    assert(ARRAY_U8[3] == 4);

    assert(ARRAY_U32[0] == 1);
    assert(ARRAY_U32[1] == 2);
    assert(ARRAY_U32[2] == 3);
}

#[inline(never)]
fn test_various_uses() {
    test_first_use();
    test_second_use();
    test_inline_use();
}

fn main() {
    test_various_uses();
}
