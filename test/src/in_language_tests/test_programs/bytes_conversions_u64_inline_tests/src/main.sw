library;

use std::{bytes::Bytes, bytes_conversions::u64::*};

#[test]
fn u64_to_be_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_be_bytes();

    assert(result.get(0).unwrap() == 8_u8);
    assert(result.get(1).unwrap() == 7_u8);
    assert(result.get(2).unwrap() == 6_u8);
    assert(result.get(3).unwrap() == 5_u8);
    assert(result.get(4).unwrap() == 4_u8);
    assert(result.get(5).unwrap() == 3_u8);
    assert(result.get(6).unwrap() == 2_u8);
    assert(result.get(7).unwrap() == 1_u8);
}

#[test]
fn u64_from_be_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(8_u8);
    bytes.push(7_u8);
    bytes.push(6_u8);
    bytes.push(5_u8);
    bytes.push(4_u8);
    bytes.push(3_u8);
    bytes.push(2_u8);
    bytes.push(1_u8);
    let result = u64::from_be_bytes(bytes);

    assert(result == 578437695752307201);
}

#[test]
fn u64_to_le_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_le_bytes();

    assert(result.get(0).unwrap() == 1_u8);
    assert(result.get(1).unwrap() == 2_u8);
    assert(result.get(2).unwrap() == 3_u8);
    assert(result.get(3).unwrap() == 4_u8);
    assert(result.get(4).unwrap() == 5_u8);
    assert(result.get(5).unwrap() == 6_u8);
    assert(result.get(6).unwrap() == 7_u8);
    assert(result.get(7).unwrap() == 8_u8);
}

#[test]
fn u64_from_le_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.push(2_u8);
    bytes.push(3_u8);
    bytes.push(4_u8);
    bytes.push(5_u8);
    bytes.push(6_u8);
    bytes.push(7_u8);
    bytes.push(8_u8);
    let result = u64::from_le_bytes(bytes);

    assert(result == 578437695752307201);
}
