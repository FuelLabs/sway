library;

// ANCHOR: to_bytes_import
use std::{bytes::Bytes, bytes_conversions::{b256::*, u16::*, u256::*, u32::*, u64::*,}};
// ANCHOR_END: to_bytes_import

pub fn convert_to_bytes() {
    // Convert any unsigned integeger to `Bytes`
    // ANCHOR: to_bytes
    let num = 5;
    let little_endian_bytes: Bytes = num.to_le_bytes();
    let big_endian_bytes: Bytes = num.to_be_bytes();
    // ANCHOR_END: to_bytes
}

pub fn convert_from_bytes() {
    let num = 5;
    let little_endian_bytes: Bytes = num.to_le_bytes();
    let big_endian_bytes: Bytes = num.to_be_bytes();

    // Convert `Bytes` to an unsigned integeger
    // ANCHOR: from_bytes
    let u16_from_le_bytes: u16 = u16::from_le_bytes(little_endian_bytes);
    let u16_from_be_bytes: u16 = u16::from_be_bytes(big_endian_bytes);

    let u32_from_le_bytes: u32 = u32::from_le_bytes(little_endian_bytes);
    let u32_from_be_bytes: u32 = u32::from_be_bytes(big_endian_bytes);

    let u64_from_le_bytes: u64 = u64::from_le_bytes(little_endian_bytes);
    let u64_from_be_bytes: u64 = u64::from_be_bytes(big_endian_bytes);

    let u256_from_le_bytes = u256::from_le_bytes(little_endian_bytes);
    let u256_from_be_bytes = u256::from_be_bytes(big_endian_bytes);

    let b256_from_le_bytes = b256::from_le_bytes(little_endian_bytes);
    let b256_from_be_bytes = b256::from_be_bytes(big_endian_bytes);
    // ANCHOR_END: from_bytes
}
