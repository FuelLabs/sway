script;

dep lib;

use lib::*;
use std::hash::sha256;

#[inline(never)]
fn test_first_use() {
    assert(C0 == true);
    assert(C1 == 42);
    assert(C2 == 0x1111111111111111111111111111111111111111111111111111111111111111);
    assert(C3.x == 42);
    assert(C3.y == true);
    assert(C4 == MyEnum::A(42));
    assert(C5 == MyEnum::B(true));
    assert(sha256(C6) == sha256("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
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
    assert(sha256(C6) == sha256("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
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
    assert(sha256(C6) == sha256("fuel"));
    assert(C7[0] == 1);
    assert(C7[1] == 2);
    assert(C7[2] == 3);
    assert(C7[3] == 4);
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
