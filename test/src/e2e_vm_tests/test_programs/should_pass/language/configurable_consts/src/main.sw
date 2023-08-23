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

fn sha256_str4(value: str[4]) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(value);
    hasher.sha256()
}

configurable {
    C0: bool = true,
    C1: u64 = 42,
    C2: b256 = 0x1111111111111111111111111111111111111111111111111111111111111111,
    C3: MyStruct = MyStruct { x: 42, y: true },
    C4: MyEnum = MyEnum::A(42),
    C5: MyEnum = MyEnum::B(true),
    C6: str[4] = "fuel",
    C7: [u64; 4] = [1, 2, 3, 4],
    C8: u64 = 0, // Unused - should not show up in the JSON file
    C9: u64 =  10 + 9 - 8 * 7 / 6 << 5 >> 4 ^ 3 | 2 & 1,
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
    assert(sha256_str4(C6) == sha256_str4("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
    assert(C9 == 23);
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
    assert(sha256_str4(C6) == sha256_str4("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
    assert(C9 == 23);
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
    assert(sha256_str4(C6) == sha256_str4("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
    assert(C9 == 23);
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
