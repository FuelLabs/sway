library;

use std::{bytes::Bytes, bytes_conversions::u32::*};

#[test]
fn u32_to_le_bytes() {
    let x: u32 = 67305985;
    let result = x.to_le_bytes();

    assert(result.get(0).unwrap() == 1_u8);
    assert(result.get(1).unwrap() == 2_u8);
    assert(result.get(2).unwrap() == 3_u8);
    assert(result.get(3).unwrap() == 4_u8);
}

#[test]
fn u32_from_le_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.push(2_u8);
    bytes.push(3_u8);
    bytes.push(4_u8);
    let result = u32::from_le_bytes(bytes);

    assert(result == 67305985_u32);
}

#[test]
fn u32_to_be_bytes() {
    let x: u32 = 67305985;
    let result = x.to_be_bytes();

    assert(result.get(0).unwrap() == 4_u8);
    assert(result.get(1).unwrap() == 3_u8);
    assert(result.get(2).unwrap() == 2_u8);
    assert(result.get(3).unwrap() == 1_u8);
}

#[test]
fn u32_from_be_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(4_u8);
    bytes.push(3_u8);
    bytes.push(2_u8);
    bytes.push(1_u8);
    let result = u32::from_be_bytes(bytes);

    assert(result == 67305985_u32);
}
