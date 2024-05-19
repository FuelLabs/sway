library;

// ANCHOR: to_u64_import
use std::primitive_conversions::u64::*;
// ANCHOR_END: to_u64_import

pub fn convert_uint_to_u64() {
    // Convert any unsigned integer to `u64`
    // ANCHOR: to_u64
    let u8_1: u8 = 2u8;
    let u16_1: u16 = 2u16;
    let u32_1: u32 = 2u32;
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;

    let u64_from_u8: u64 = u8_1.as_u64();

    let u64_from_u16: u64 = u16_1.as_u64();

    let u64_from_u32: u64 = u32_1.as_u64();

    let u64_from_u256: Option<u64> = <u64 as TryFrom<u256>>::try_from(u256_1);
    // ANCHOR_END: to_u64
}
