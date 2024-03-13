library;

// ANCHOR: to_u16_import
use std::{primitive_conversions::{u16::*, u32::*, u64::*,},};
// ANCHOR_END: to_u16_import

pub fn convert_uint_to_u16() {
    // Convert any unsigned integer to `u16`
    // ANCHOR: to_u16
    let u8_1: u8 = 2u8;
    let u32_1: u32 = 2u32;
    let u64_1: u64 = 2;
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;

    let u16_from_u8: u16 = u8_1.as_u16();

    let u16_from_u32_1: Option<u16> = u32_1.try_as_u16();
    let u16_from_u32_2: Option<u16> = <u16 as TryFrom<u32>>::try_from(u32_1);

    let u16_from_u64_1: Option<u16> = u64_1.try_as_u16();
    let u16_from_u64_2: Option<u16> = <u16 as TryFrom<u64>>::try_from(u64_1);

    let u16_from_u256: Option<u16> = <u16 as TryFrom<u256>>::try_from(u256_1);
    // ANCHOR_END: to_u16
}
