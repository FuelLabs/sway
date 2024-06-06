library;

use std::array_conversions::b256::*;

#[test]
fn b256_from_le_bytes() {
    let bytes = [
        32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8, 22_u8,
        21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8, 12_u8, 11_u8,
        10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8, 2_u8, 1_u8,
    ];

    let x = b256::from_le_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn b256_to_le_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_le_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes[i.as_u64()] == 32_u8 - i);
        i += 1_u8;
    }
}

#[test]
fn b256_from_be_bytes() {
    let bytes = [
        1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8, 11_u8, 12_u8,
        13_u8, 14_u8, 15_u8, 16_u8, 17_u8, 18_u8, 19_u8, 20_u8, 21_u8, 22_u8, 23_u8,
        24_u8, 25_u8, 26_u8, 27_u8, 28_u8, 29_u8, 30_u8, 31_u8, 32_u8,
    ];

    let x = b256::from_be_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn b256_to_be_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_be_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes[i.as_u64()] == i + 1_u8);
        i += 1_u8;
    }
}
