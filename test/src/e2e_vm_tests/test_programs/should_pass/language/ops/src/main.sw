script;

use core::ops::*;

// 0b0000_1111 = 15
// 0b0101_0101 = 85
// 0b1010_1010 = 170
// 0b1111_0000 = 240
// 0b1111_1111 = 255

fn u64_ops() -> bool {
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
    assert(max << 64 == 0);

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
    assert(max >> 64 == 0);

    true
}

fn u32_ops() -> bool {
    let max = 4294967295_u32; // 0b11111111_11111111_11111111_11111111
    let A = 2172748161_u32; // 0b10000001_10000001_10000001_10000001
    let B = 2122219134_u32; // 0b01111110_01111110_01111110_01111110
    let C = 50529026_u32; // 0b00000011_00000011_00000011_00000010
    let D = 1086374080_u32; // 0b01000000_11000000_11000000_11000000
    let E = 2147483648_u32; // 0b10000000_00000000_00000000_00000000

    assert(0_u32 & 0_u32 == 0_u32);
    assert(0_u32 & 1_u32 == 0_u32);
    assert(1_u32 & 1_u32 == 1_u32);
    assert(15_u32 & 255_u32 == 15_u32);
    assert(15_u32 & 85_u32 == 5_u32);
    assert(240_u32 & 255_u32 == 240_u32);
    assert(85_u32 & 170_u32 == 0_u32);
    assert(0_u32 & max == 0_u32);
    assert(max & max == max);
    assert(max & A == A);
    assert(max & B == B);
    assert(A & B == 0_u32);

    assert(0_u32 | 0_u32 == 0_u32);
    assert(0_u32 | 1_u32 == 1_u32);
    assert(1_u32 | 1_u32 == 1_u32);
    assert(15_u32 | 240_u32 == 255_u32);
    assert(240_u32 | 170_u32 == 250_u32);
    assert(15_u32 | 170_u32 == 175_u32);
    assert(15_u32 | 255_u32 == 255_u32);
    assert(max | 0_u32 == max);
    assert(A | B == max);
    assert(A | 0_u32 == A);
    assert(B | 0_u32 == B);

    assert(0_u32 ^ 0_u32 == 0_u32);
    assert(0_u32 ^ 1_u32 == 1_u32);
    assert(1_u32 ^ 1_u32 == 0_u32);
    assert(15_u32 ^ 240_u32 == 255_u32);
    assert(85_u32 ^ 170_u32 == 255_u32);
    assert(85_u32 ^ 85_u32 == 0_u32);
    assert(240_u32 ^ 255_u32 == 15_u32);
    assert(max ^ 0_u32 == max);
    assert(max ^ A == B);
    assert(max ^ B == A);
    assert(A ^ B == max);

    assert(0_u32 << 0 == 0_u32);
    assert(0_u32 << 1 == 0_u32);
    assert(1_u32 << 1 == 2_u32);
    assert(1_u32 << 1 == 2_u32);
    assert(2_u32 << 1 == 4_u32);
    assert(255_u32 << 21 == 534773760_u32);
    assert(max << 1 & max == 4294967294_u32);
    assert(max << 2 & max == 4294967292_u32);
    assert(A << 1 & max == C);
    assert(max << 31 & max == E);
    assert(max << 32 & max == 0_u32);

    assert(0_u32 >> 0 == 0_u32);
    assert(0_u32 >> 1 == 0_u32);
    assert(1_u32 >> 1 == 0_u32);
    assert(1_u32 >> 2 == 0_u32);
    assert(2_u32 >> 1 == 1_u32);
    assert(2_u32 >> 2 == 0_u32);
    assert(8_u32 >> 2 == 2_u32);
    assert(255_u32 >> 1 == 127_u32);
    assert(255_u32 >> 3 == 31_u32);
    assert(A >> 1 == D);
    assert(A >> 21 == 1036_u32);
    assert(max >> 1 == 2147483647_u32);
    assert(max >> 31 & max == 1_u32);
    assert(max >> 32 & max == 0_u32);

    true
}

fn u16_ops() -> bool {
    let max = 65535_u16; // 0b11111111_11111111
    let A = 33153_u16; // 0b10000001_10000001
    let B = 32382_u16; // 0b01111110_01111110
    let C = 770_u16; // 0b00000011_00000010
    let D = 16576_u16; // 0b01000000_11000000
    let E = 32768_u16; // 0b10000000_00000000

    assert(0_u16 & 0_u16 == 0_u16);
    assert(0_u16 & 1_u16 == 0_u16);
    assert(1_u16 & 1_u16 == 1_u16);
    assert(15_u16 & 255_u16 == 15_u16);
    assert(15_u16 & 85_u16 == 5_u16);
    assert(240_u16 & 255_u16 == 240_u16);
    assert(85_u16 & 170_u16 == 0_u16);
    assert(0_u16 & max == 0_u16);
    assert(max & max == max);
    assert(max & A == A);
    assert(max & B == B);
    assert(A & B == 0_u16);

    assert(0_u16 | 0_u16 == 0_u16);
    assert(0_u16 | 1_u16 == 1_u16);
    assert(1_u16 | 1_u16 == 1_u16);
    assert(15_u16 | 240_u16 == 255_u16);
    assert(240_u16 | 170_u16 == 250_u16);
    assert(15_u16 | 170_u16 == 175_u16);
    assert(15_u16 | 255_u16 == 255_u16);
    assert(max | 0_u16 == max);
    assert(A | B == max);
    assert(A | 0_u16 == A);
    assert(B | 0_u16 == B);

    assert(0_u16 ^ 0_u16 == 0_u16);
    assert(0_u16 ^ 1_u16 == 1_u16);
    assert(1_u16 ^ 1_u16 == 0_u16);
    assert(15_u16 ^ 240_u16 == 255_u16);
    assert(85_u16 ^ 170_u16 == 255_u16);
    assert(85_u16 ^ 85_u16 == 0_u16);
    assert(240_u16 ^ 255_u16 == 15_u16);
    assert(max ^ 0_u16 == max);
    assert(max ^ A == B);
    assert(max ^ B == A);
    assert(A ^ B == max);

    assert(0_u16 << 0 == 0_u16);
    assert(0_u16 << 1 == 0_u16);
    assert(1_u16 << 1 == 2_u16);
    assert(1_u16 << 1 == 2_u16);
    assert(2_u16 << 1 == 4_u16);
    assert(255_u16 << 4 == 4080_u16);
    assert(max << 1 & max == 65534_u16);
    assert(max << 2 & max == 65532_u16);
    assert(A << 1 & max == C);
    assert(max << 15 & max == E);
    assert(max << 16 & max == 0_u16);

    assert(0_u16 >> 0 == 0_u16);
    assert(0_u16 >> 1 == 0_u16);
    assert(1_u16 >> 1 == 0_u16);
    assert(1_u16 >> 2 == 0_u16);
    assert(2_u16 >> 1 == 1_u16);
    assert(2_u16 >> 2 == 0_u16);
    assert(8_u16 >> 2 == 2_u16);
    assert(255_u16 >> 1 == 127_u16);
    assert(255_u16 >> 3 == 31_u16);
    assert(A >> 1 == D);
    assert(A >> 4 == 2072_u16);
    assert(max >> 1 == 32767_u16);
    assert(max >> 15 & max == 1_u16);
    assert(max >> 16 & max == 0_u16);

    true
}

fn u8_ops() -> bool {
    let max = 255_u8; // 0b11111111
    let A = 129_u8; // 0b10000001
    let B = 126_u8; // 0b01111110
    let C = 2_u8; // 0b00000010
    let D = 64_u8; // 0b01000000
    let E = 128_u8; // 0b10000000

    assert(0_u8 & 0_u8 == 0_u8);
    assert(0_u8 & 1_u8 == 0_u8);
    assert(1_u8 & 1_u8 == 1_u8);
    assert(15_u8 & 255_u8 == 15_u8);
    assert(15_u8 & 85_u8 == 5_u8);
    assert(240_u8 & 255_u8 == 240_u8);
    assert(85_u8 & 170_u8 == 0_u8);
    assert(0_u8 & max == 0_u8);
    assert(max & max == max);
    assert(max & A == A);
    assert(max & B == B);
    assert(A & B == 0_u8);

    assert(0_u8 | 0_u8 == 0_u8);
    assert(0_u8 | 1_u8 == 1_u8);
    assert(1_u8 | 1_u8 == 1_u8);
    assert(15_u8 | 240_u8 == 255_u8);
    assert(240_u8 | 170_u8 == 250_u8);
    assert(15_u8 | 170_u8 == 175_u8);
    assert(15_u8 | 255_u8 == 255_u8);
    assert(max | 0_u8 == max);
    assert(A | B == max);
    assert(A | 0_u8 == A);
    assert(B | 0_u8 == B);

    assert(0_u8 ^ 0_u8 == 0_u8);
    assert(0_u8 ^ 1_u8 == 1_u8);
    assert(1_u8 ^ 1_u8 == 0_u8);
    assert(15_u8 ^ 240_u8 == 255_u8);
    assert(85_u8 ^ 170_u8 == 255_u8);
    assert(85_u8 ^ 85_u8 == 0_u8);
    assert(240_u8 ^ 255_u8 == 15_u8);
    assert(max ^ 0_u8 == max);
    assert(max ^ A == B);
    assert(max ^ B == A);
    assert(A ^ B == max);

    assert(0_u8 << 0 == 0_u8);
    assert(0_u8 << 1 == 0_u8);
    assert(1_u8 << 1 == 2_u8);
    assert(1_u8 << 1 == 2_u8);
    assert(2_u8 << 1 == 4_u8);
    assert(31_u8 << 2 == 124_u8);
    assert(max << 1 & max == 254_u8);
    assert(max << 2 & max == 252_u8);
    assert(A << 1 & max == C);
    assert(max << 7 & max == E);
    assert(max << 8 & max == 0_u8);

    assert(0_u8 >> 0 == 0_u8);
    assert(0_u8 >> 1 == 0_u8);
    assert(1_u8 >> 1 == 0_u8);
    assert(1_u8 >> 2 == 0_u8);
    assert(2_u8 >> 1 == 1_u8);
    assert(2_u8 >> 2 == 0_u8);
    assert(8_u8 >> 2 == 2_u8);
    assert(255_u8 >> 1 == 127_u8);
    assert(255_u8 >> 3 == 31_u8);
    assert(A >> 1 == D);
    assert(A >> 4 == 8_u8);
    assert(max >> 1 == 127_u8);
    assert(max >> 7 & max == 1_u8);
    assert(max >> 8 & max == 0_u8);

    true
}

fn main() -> bool {
    u64_ops() && u32_ops() && u16_ops() && u8_ops()
}
