library;

use std::{bytes::Bytes, bytes_conversions::b256::*};

#[test]
fn b256_from_le_bytes() {
    let mut bytes = Bytes::with_capacity(32);
    let mut i: u8 = 0;
    while i < 32_u8 {
        bytes.push(32_u8 - i);
        i += 1_u8;
    }

    let x = b256::from_le_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn b256_to_le_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_le_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes.get(i.as_u64()).unwrap() == 32_u8 - i);
        i += 1_u8;
    }
}

#[test]
fn b256_from_be_bytes() {
    let mut bytes = Bytes::with_capacity(32);

    let mut i: u8 = 0;
    while i < 32_u8 {
        bytes.push(i + 1_u8);
        i += 1_u8;
    }

    let x = b256::from_be_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn b256_to_be_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_be_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes.get(i.as_u64()).unwrap() == i + 1_u8);
        i += 1_u8;
    }
}
