library;

use std::{primitive_conversions::u32::*, u128::U128};

#[test]
fn u32_from_u8() {
    let u8_1: u8 = u8::min();
    let u8_2: u8 = 1u8;
    let u8_3: u8 = u8::max();

    let u32_1 = u32::from(u8_1);
    let u32_2 = u32::from(u8_2);
    let u32_3 = u32::from(u8_3);

    assert(u32_1 == 0u32);
    assert(u32_2 == 1u32);
    assert(u32_3 == 255u32);
}

#[test]
fn u32_into_u8() {
    let u8_1: u8 = u8::min();
    let u8_2: u8 = 1u8;
    let u8_3: u8 = u8::max();

    let u32_1: u32 = u8_1.into();
    let u32_2: u32 = u8_2.into();
    let u32_3: u32 = u8_3.into();

    assert(u32_1 == 0u32);
    assert(u32_2 == 1u32);
    assert(u32_3 == 255u32);
}

#[test]
fn u32_from_u16() {
    let u16_1: u16 = u16::min();
    let u16_2: u16 = 1u16;
    let u16_3: u16 = u16::max();

    let u32_1 = u32::from(u16_1);
    let u32_2 = u32::from(u16_2);
    let u32_3 = u32::from(u16_3);

    assert(u32_1 == 0u32);
    assert(u32_2 == 1u32);
    assert(u32_3 == 65535u32);
}

#[test]
fn u32_into_u16() {
    let u16_1: u16 = u16::min();
    let u16_2: u16 = 1u16;
    let u16_3: u16 = u16::max();

    let u32_1: u32 = u16_1.into();
    let u32_2: u32 = u16_2.into();
    let u32_3: u32 = u16_3.into();

    assert(u32_1 == 0u32);
    assert(u32_2 == 1u32);
    assert(u32_3 == 65535u32);
}

#[test]
fn u32_try_from_u64() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = 2u64;
    let u64_3: u64 = u32::max().as_u64();
    let u64_4: u64 = u32::max().as_u64() + 1;

    let u32_1 = u32::try_from(u64_1);
    let u32_2 = u32::try_from(u64_2);
    let u32_3 = u32::try_from(u64_3);
    let u32_4 = u32::try_from(u64_4);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 0u32);

    assert(u32_2.is_some());
    assert(u32_2.unwrap() == 2u32);

    assert(u32_3.is_some());
    assert(u32_3.unwrap() == u32::max());

    assert(u32_4.is_none());
}

#[test]
fn u32_try_from_u256() {
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000000u256;
    let u256_2: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_3: u256 = u32::max().as_u256();
    let u256_4: u256 = u32::max().as_u256() + 1;

    let u32_1 = u32::try_from(u256_1);
    let u32_2 = u32::try_from(u256_2);
    let u32_3 = u32::try_from(u256_3);
    let u32_4 = u32::try_from(u256_4);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 0u32);

    assert(u32_2.is_some());
    assert(u32_2.unwrap() == 2u32);

    assert(u32_3.is_some());
    assert(u32_3.unwrap() == u32::max());

    assert(u32_4.is_none());
}

#[test]
fn u32_try_from_u128() {
    let u128_1: U128 = U128::new();
    let u128_2: U128 = U128::from((0u64, 2u32.as_u64()));
    let u128_3: U128 = U128::from((0u64, u32::max().as_u64()));
    let u128_4: U128 = U128::from((0, u32::max().as_u64() + 1));

    let u32_1 = u32::try_from(u128_1);
    let u32_2 = u32::try_from(u128_2);
    let u32_3 = u32::try_from(u128_3);
    let u32_4 = u32::try_from(u128_4);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 0u32);

    assert(u32_2.is_some());
    assert(u32_2.unwrap() == 2u32);

    assert(u32_3.is_some());
    assert(u32_3.unwrap() == u32::max());

    assert(u32_4.is_none());
}
