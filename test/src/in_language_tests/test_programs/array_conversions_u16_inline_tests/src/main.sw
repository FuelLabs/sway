library;

use std::array_conversions::u16::*;

#[test]
fn u16_to_le_bytes() {
    let x: u16 = 513;
    let result = x.to_le_bytes();

    assert(result[0] == 1_u8);
    assert(result[1] == 2_u8);
}

#[test]
fn u16_from_le_bytes() {
    let bytes = [1_u8, 2_u8];
    let result = u16::from_le_bytes(bytes);

    assert(result == 513_u16);
}

#[test]
fn u16_to_be_bytes() {
    let x: u16 = 513;
    let result = x.to_be_bytes();

    assert(result[0] == 2_u8);
    assert(result[1] == 1_u8);
}

#[test]
fn u16_from_be_bytes() {
    let bytes = [2_u8, 1_u8];
    let result = u16::from_be_bytes(bytes);

    assert(result == 513_u16);
}
