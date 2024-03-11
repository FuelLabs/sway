library;

// ANCHOR: to_u32_import
use std::{primitive_conversions::{u32::*, u64::*,},};
// ANCHOR_END: to_u32_import

pub fn convert_uint_to_u32() {
    // Convert any unsigned integer to `u32`
    // ANCHOR: to_u32
    let u8_1: u8 = 2u8;
    let u16_1: u16 = 2u16;
    let u64_1: u64 = 2;
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;

    let u32_from_u8: u32 = u8_1.as_u32();

    let u32_from_u16: u32 = u16_1.as_u32();

    let u32_from_u64_1: Option<u32> = u64_1.try_as_u32();
    let u32_from_u64_2: Option<u32> = <u32 as TryFrom<u64>>::try_from(u64_1);

    let u32_from_u256: Option<u32> = <u32 as TryFrom<u256>>::try_from(u256_1);
    // ANCHOR_END: to_u32
}
