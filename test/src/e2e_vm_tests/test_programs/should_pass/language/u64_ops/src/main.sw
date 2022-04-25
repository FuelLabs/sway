script;

use core::ops::*;
use std::assert::assert;
use std::chain::log_u64;

fn main() -> bool {
    // 0b0000_1111 = 15
    // 0b0101_0101 = 85
    // 0b1010_1010 = 170
    // 0b1111_0000 = 240
    // 0b1111_1111 = 255
    let max = 18446744073709551615; // 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111
    let A = 9331882296111890817; // 0b10000001_10000001_10000001_10000001_10000001_10000001_10000001_10000001
    let B = 9114861777597660798; // 0b01111110_01111110_01111110_01111110_01111110_01111110_01111110_01111110
    let C = 217020518514230018; // 0b00000011_00000011_00000011_00000011_00000011_00000011_00000011_00000010
    let D = 4665941148055945408; // 0b01000000_11000000_11000000_11000000_11000000_11000000_11000000_11000000
    let E = 9223372036854775808; // 0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000

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

    assert(0 ^ 0 == 0);
    assert(0 ^ 1 == 1);
    assert(1 ^ 1 == 0);
    assert(15 ^ 240 == 255);
    assert(85 ^ 170 == 255);
    assert(85 ^ 85 == 0);
    assert(240 ^ 255 == 15);
    assert(max ^ 0 == max);
    assert(max ^ A == B);
    assert(max ^ B == A);
    assert(A ^ B == max);

    assert(0 << 0 == 0);
    assert(0 << 1 == 0);
    assert(1 << 1 == 2);
    assert(1 << 1 == 2);
    assert(2 << 1 == 4);
    assert(255 << 42 == 1121501860331520);
    assert(max << 1 == 18446744073709551614);
    assert(max << 2 == 18446744073709551612);
    assert(A << 1 == C);
    assert(max << 63 == E);
    // this will break when vm is brought in line with spec re: wrapping
    // https://github.com/FuelLabs/fuel-vm/issues/104
    // will be 0 or panic
    assert(max << 64 == max);

    assert(0 >> 0 == 0);
    assert(0 >> 1 == 0);
    assert(1 >> 1 == 0);
    assert(1 >> 2 == 0);
    assert(2 >> 1 == 1);
    assert(2 >> 2 == 0);
    assert(8 >> 2 == 2);
    assert(255 >> 1 == 127);
    assert(255 >> 3 == 31);
    assert(A >> 1 == D);
    assert(A >> 42 == 2121824);
    assert(max >> 1 == 9223372036854775807);
    assert(max >> 63 == 1);
    // this will break when vm is brought in line with spec re: wrapping
    // https://github.com/FuelLabs/fuel-vm/issues/104
    // will be 0 or panic
    assert(max >> 64 == max);

    true
}
