library;

// ANCHOR: to_byte_array_import
use std::array_conversions::{b256::*, u16::*, u256::*, u32::*, u64::*,};
// ANCHOR_END: to_byte_array_import

pub fn to_byte_array() {
    // ANCHOR: to_byte_array
    let u16_1: u16 = 2u16;
    let u32_1: u32 = 2u32;
    let u64_1: u64 = 2u64;
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let b256_1: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    // little endian
    let le_byte_array_from_u16: [u8; 2] = u16_1.to_le_bytes();
    let le_byte_array_from_u32: [u8; 4] = u32_1.to_le_bytes();
    let le_byte_array_from_u64: [u8; 8] = u64_1.to_le_bytes();
    let le_byte_array_from_u256: [u8; 32] = u256_1.to_le_bytes();
    let le_byte_array_from_b256: [u8; 32] = b256_1.to_le_bytes();
    // big endian
    let be_byte_array_from_u16: [u8; 2] = u16_1.to_be_bytes();
    let be_byte_array_from_u32: [u8; 4] = u32_1.to_be_bytes();
    let be_byte_array_from_u64: [u8; 8] = u64_1.to_be_bytes();
    let be_byte_array_from_u256: [u8; 32] = u256_1.to_be_bytes();
    let be_byte_array_from_b256: [u8; 32] = b256_1.to_be_bytes();
    // ANCHOR_END: to_byte_array
}
pub fn from_byte_array() {
    // ANCHOR: from_byte_array
    let u16_byte_array: [u8; 2] = [2_u8, 1_u8];
    let u32_byte_array: [u8; 4] = [4_u8, 3_u8, 2_u8, 1_u8];
    let u64_byte_array: [u8; 8] = [8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8, 2_u8, 1_u8];
    let u256_byte_array: [u8; 32] = [
        32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8, 22_u8,
        21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8, 12_u8, 11_u8,
        10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8, 2_u8, 1_u8,
    ];
    // little endian
    let le_u16_from_byte_array: u16 = u16::from_le_bytes(u16_byte_array);
    let le_u32_from_byte_array: u32 = u32::from_le_bytes(u32_byte_array);
    let le_u64_from_byte_array: u64 = u64::from_le_bytes(u64_byte_array);
    let le_u256_from_byte_array: u256 = u256::from_le_bytes(u256_byte_array);
    let le_b256_from_byte_array: b256 = b256::from_le_bytes(u256_byte_array);
    // big endian
    let be_u16_from_byte_array: u16 = u16::from_be_bytes(u16_byte_array);
    let be_u32_from_byte_array: u32 = u32::from_be_bytes(u32_byte_array);
    let be_u64_from_byte_array: u64 = u64::from_be_bytes(u64_byte_array);
    let be_u256_from_byte_array: u256 = u256::from_be_bytes(u256_byte_array);
    let be_b256_from_byte_array: b256 = b256::from_be_bytes(u256_byte_array);
    // ANCHOR_END: from_byte_array
}
