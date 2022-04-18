script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    // 0b0000_1111 = 15
    // 0b0101_0101 = 85
    // 0b1010_1010 = 170
    // 0b1111_0000 = 240
    // 0b1111_1111 = 255

    assert(~u64::binary_and(0, 0) == 0);
    assert(~u64::binary_and(0, 1) == 0);
    assert(~u64::binary_and(1, 1) == 1);
    assert(~u64::binary_and(15, 255) == 15);
    assert(~u64::binary_and(15, 85) == 5);
    assert(~u64::binary_and(240, 255) == 240);
    assert(~u64::binary_and(85, 170) == 0);

    assert(~u64::binary_or(0, 0) == 0);
    assert(~u64::binary_or(0, 1) == 1);
    assert(~u64::binary_or(1, 1) == 1);
    assert(~u64::binary_or(15, 240) == 255);
    assert(~u64::binary_or(240, 170) == 250);
    assert(~u64::binary_or(15, 170) == 175);
    assert(~u64::binary_or(15, 255) == 255);

    assert(~u64::binary_xor(0, 0) == 0);
    assert(~u64::binary_xor(0, 1) == 1);
    assert(~u64::binary_xor(1, 1) == 0);
    assert(~u64::binary_xor(15, 240) == 255);
    assert(~u64::binary_xor(85, 170) == 255);
    assert(~u64::binary_xor(85, 85) == 0);
    assert(~u64::binary_xor(240, 255) == 15);

    true
}
