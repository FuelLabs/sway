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
    assert(255_u16 << 4 == 4080_u16); // here
    assert(max << 1 & max == 65534_u16); // here
    assert(max << 2 & max == 65532_u16); // here
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
    assert(A >> 4 == 1036_u16); // here
    assert(max >> 1 == 32767_u16); // here
    assert(max >> 15 & max == 1_u16); // here
    assert(max >> 16 & max == 0_u16); // here