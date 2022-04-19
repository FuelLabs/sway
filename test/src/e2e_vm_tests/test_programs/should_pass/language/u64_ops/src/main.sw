script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    // 0b0000_1111 = 15
    // 0b0101_0101 = 85
    // 0b1010_1010 = 170
    // 0b1111_0000 = 240
    // 0b1111_1111 = 255
    let max = 18446744073709551615; // 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111
    let A = 9331882296111890817;    // 0b10000001_10000001_10000001_10000001_10000001_10000001_10000001_10000001
    let B = 9114861777597660798;    // 0b01111110_01111110_01111110_01111110_01111110_01111110_01111110_01111110

    assert(~u64::binary_and(0, 0) == 0);
    assert(~u64::binary_and(0, 1) == 0);
    assert(~u64::binary_and(1, 1) == 1);
    assert(~u64::binary_and(15, 255) == 15);
    assert(~u64::binary_and(15, 85) == 5);
    assert(~u64::binary_and(240, 255) == 240);
    assert(~u64::binary_and(85, 170) == 0);
    assert(~u64::binary_and(0, max) == 0);
    assert(~u64::binary_and(max, max) == max);
    assert(~u64::binary_and(max, A) == A);
    assert(~u64::binary_and(max, B) == B);
    assert(~u64::binary_and(A, B) == 0);


    assert(~u64::binary_or(0, 0) == 0);
    assert(~u64::binary_or(0, 1) == 1);
    assert(~u64::binary_or(1, 1) == 1);
    assert(~u64::binary_or(15, 240) == 255);
    assert(~u64::binary_or(240, 170) == 250);
    assert(~u64::binary_or(15, 170) == 175);
    assert(~u64::binary_or(15, 255) == 255);
    assert(~u64::binary_or(max, 0) == max);
    assert(~u64::binary_or(A, B) == max);
    assert(~u64::binary_or(A, 0) == A);
    assert(~u64::binary_or(B, 0) == B);

    assert(~u64::binary_xor(0, 0) == 0);
    assert(~u64::binary_xor(0, 1) == 1);
    assert(~u64::binary_xor(1, 1) == 0);
    assert(~u64::binary_xor(15, 240) == 255);
    assert(~u64::binary_xor(85, 170) == 255);
    assert(~u64::binary_xor(85, 85) == 0);
    assert(~u64::binary_xor(240, 255) == 15);
    assert(~u64::binary_xor(max, 0) == max);
    assert(~u64::binary_xor(max, A) == B);
    assert(~u64::binary_xor(max, B) == A);
    assert(~u64::binary_xor(A, B) == max);

    true
}
