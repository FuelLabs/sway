library;

use std::{bytes::Bytes, bytes_conversions::u16::*};

#[test]
fn u16_to_le_bytes() {
    let x: u16 = 513;
    let result = x.to_le_bytes();

    assert(result.get(0).unwrap() == 1_u8);
    assert(result.get(1).unwrap() == 2_u8);
}

#[test]
fn u16_from_le_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.push(2_u8);
    let result = u16::from_le_bytes(bytes);

    assert(result == 513_u16);
}

#[test]
fn u16_to_be_bytes() {
    let x: u16 = 513;
    let result = x.to_be_bytes();

    assert(result.get(0).unwrap() == 2_u8);
    assert(result.get(1).unwrap() == 1_u8);
}

#[test]
fn u16_from_be_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(2_u8);
    bytes.push(1_u8);
    let result = u16::from_be_bytes(bytes);

    assert(result == 513_u16);
}
