library;

use std::{primitive_conversions::u64::*, u128::U128};

#[test]
fn u64_from_u8() {
    let u8_1: u8 = 0u8;
    let u8_2: u8 = 2u8;
    let u8_3: u8 = u8::max();

    let u64_1 = u64::from(u8_1);
    let u64_2 = u64::from(u8_2);
    let u64_3 = u64::from(u8_3);

    assert(u64_1 == 0u64);
    assert(u64_2 == 2u64);
    assert(u64_3 == 255u64);
}

#[test]
fn u64_into_u8() {
    let u8_1: u8 = 0u8;
    let u8_2: u8 = 2u8;
    let u8_3: u8 = u8::max();

    let u64_1: u64 = u8_1.into();
    let u64_2: u64 = u8_2.into();
    let u64_3: u64 = u8_3.into();

    assert(u64_1 == 0u64);
    assert(u64_2 == 2u64);
    assert(u64_3 == 255u64);
}

#[test]
fn u64_from_u16() {
    let u16_1: u16 = u16::min();
    let u16_2: u16 = 2u16;
    let u16_3: u16 = u16::max();

    let u64_1 = u64::from(u16_1);
    let u64_2 = u64::from(u16_2);
    let u64_3 = u64::from(u16_3);

    assert(u64_1 == 0u64);
    assert(u64_2 == 2u64);
    assert(u64_3 == 65535u64);
}

#[test]
fn u64_into_u16() {
    let u16_1: u16 = u16::min();
    let u16_2: u16 = 2u16;
    let u16_3: u16 = u16::max();

    let u64_1: u64 = u16_1.into();
    let u64_2: u64 = u16_2.into();
    let u64_3: u64 = u16_3.into();

    assert(u64_1 == 0u64);
    assert(u64_2 == 2u64);
    assert(u64_3 == 65535u64);
}

#[test]
fn u64_from_u32() {
    let u32_1: u32 = 0u32;
    let u32_2: u32 = 2u32;
    let u32_3: u32 = u32::max();

    let u64_1 = u64::from(u32_1);
    let u64_2 = u64::from(u32_2);
    let u64_3 = u64::from(u32_3);

    assert(u64_1 == 0u64);
    assert(u64_2 == 2u64);
    assert(u64_3 == 4294967295u64);
}

#[test]
fn u64_into_u32() {
    let u32_1: u32 = 0u32;
    let u32_2: u32 = 2u32;
    let u32_3: u32 = u32::max();

    let u64_1: u64 = u32_1.into();
    let u64_2: u64 = u32_2.into();
    let u64_3: u64 = u32_3.into();

    assert(u64_1 == 0u64);
    assert(u64_2 == 2u64);
    assert(u64_3 == 4294967295u64);
}

#[test]
fn u64_try_from_u256() {
    let u256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000u256;
    let u256_2 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_3 = u64::max().as_u256();
    let u256_4 = u64::max().as_u256() + 1;

    let u64_1 = u64::try_from(u256_1);
    let u64_2 = u64::try_from(u256_2);
    let u64_3 = u64::try_from(u256_3);
    let u64_4 = u64::try_from(u256_4);

    assert(u64_1.is_some());
    assert(u64_1.unwrap() == 0);

    assert(u64_2.is_some());
    assert(u64_2.unwrap() == 2);

    assert(u64_3.is_some());
    assert(u64_3.unwrap() == u64::max());

    assert(u64_4.is_none());
}

#[test]
fn u64_try_from_u128() {
    let u128_1: U128 = U128::new();
    let u128_2: U128 = U128::from((0, 2));
    let u128_3: U128 = U128::from((0, u64::max()));
    let u128_4: U128 = U128::from((1, 0));

    let u64_1 = u64::try_from(u128_1);
    let u64_2 = u64::try_from(u128_2);
    let u64_3 = u64::try_from(u128_3);
    let u64_4 = u64::try_from(u128_4);

    assert(u64_1.is_some());
    assert(u64_1.unwrap() == 0u64);

    assert(u64_2.is_some());
    assert(u64_2.unwrap() == 2u64);

    assert(u64_3.is_some());
    assert(u64_3.unwrap() == u64::max());

    assert(u64_4.is_none());
}
