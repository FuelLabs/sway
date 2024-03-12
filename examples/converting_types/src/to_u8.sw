library;

// ANCHOR: to_u8_import
use std::{primitive_conversions::{u16::*, u32::*, u64::*, u8::*,},};
// ANCHOR_END: to_u8_import

pub fn convert_uint_to_u8() {
    // Convert any unsigned integer to `u8`
    // ANCHOR: to_u8
    let u16_1: u16 = 2u16;
    let u32_1: u32 = 2u32;
    let u64_1: u64 = 2;
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;

    let u8_from_u16_1: Option<u8> = u16_1.try_as_u8();
    let u8_from_u16_2: Option<u8> = <u8 as TryFrom<u16>>::try_from(u16_1);

    let u8_from_u32_1: Option<u8> = u32_1.try_as_u8();
    let u8_from_u32_2: Option<u8> = <u8 as TryFrom<u32>>::try_from(u32_1);

    let u8_from_u64_1: Option<u8> = u64_1.try_as_u8();
    let u8_from_u64_2: Option<u8> = <u8 as TryFrom<u64>>::try_from(u64_1);

    let u8_from_u256: Option<u8> = <u8 as TryFrom<u256>>::try_from(u256_1);
    // ANCHOR_END: to_u8
    assert(u8_from_u16_1.unwrap() == 2u8);
    assert(u8_from_u16_2.unwrap() == 2u8);
    assert(u8_from_u32_1.unwrap() == 2u8);
    assert(u8_from_u32_2.unwrap() == 2u8);
    assert(u8_from_u64_1.unwrap() == 2u8);
    assert(u8_from_u64_2.unwrap() == 2u8);
    assert(u8_from_u256.unwrap() == 2u8);
}
