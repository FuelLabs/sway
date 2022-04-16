script;

use core::ops::{binary_and, binary_or, binary_xor};
use std::assert::assert;

fn main() -> bool {

    // 0b0000_1111 = 15
    // 0b0101_0101 = 85
    // 0b1010_1010 = 170
    // 0b1111_0000 = 240
    // 0b1111_1111 = 255

    assert(0.binary_and(0) == 0);
    assert(0.binary_and(1) == 0);
    assert(1.binary_and(1) == 1);
    assert(15.binary_and(255) == 15);
    assert(15.binary_and(85) == 5);
    assert(240.binary_and(255) == 240);
    assert(85.binary_and(170) == 0);

    assert(0.binary_or(0) == 0);
    assert(0.binary_or(1) == 1);
    assert(1.binary_or(1) == 1);
    assert(15.binary_or(240) == 255);
    assert(240.binary_or(170) == 250);
    assert(15.binary_or(170) == 175);
    assert(15.binary_or(255) == 255);

    assert(0.binary_xor(0) == 0);
    assert(0.binary_xor(1) == 1);
    assert(1.binary_xor(1) == 0);
    assert(15.binary_xor(240) == 255);
    assert(85.binary_xor(170) == 255);
    assert(85.binary_xor(85) == 0);
    assert(240.binary_xor(255) == 15);

    true
}
