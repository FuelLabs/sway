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
    let A = 9331882296111890817; // 0b10000001_10000001_10000001_10000001_10000001_10000001_10000001_10000001
    let B = 9114861777597660798; // 0b01111110_01111110_01111110_01111110_01111110_01111110_01111110_01111110
    assert(0 & 0 == 0);
    assert(0 & 1 == 0);
    assert(1 & 1 == 1);
    assert(15 & 255 == 15);
    assert(15 & 85 == 5);
    assert(240 & 255 == 240);
    assert(85 & 170 == 0);
    assert(0 & max == 0);
    assert(max & max == max);
    assert(max & A == A);
    assert(max & B == B);
    assert(A & B == 0);

    assert(0 | 0 == 0);
    assert(0 | 1 == 1);
    assert(1 | 1 == 1);
    assert(15 | 240 == 255);
    assert(240 | 170 == 250);
    assert(15 | 170 == 175);
    assert(15 | 255 == 255);
    assert(max | 0 == max);
    assert(A | B == max);
    assert(A | 0 == A);
    assert(B | 0 == B);

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
