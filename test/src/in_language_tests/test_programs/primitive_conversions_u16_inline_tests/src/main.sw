library;

use std::{primitive_conversions::u16::*, u128::U128};

#[test]
fn u16_from_u8() {
    let u8_1: u8 = u8::min();
    let u8_2: u8 = 255u8;
    let u8_3: u8 = 2u8;

    let u16_1 = u16::from(u8_1);
    let u16_2 = u16::from(u8_2);
    let u16_3 = u16::from(u8_3);

    assert(u16_1 == 0u16);
    assert(u16_2 == 255u16);
    assert(u16_3 == 2u16);
}

#[test]
fn u16_into_u8() {
    let u8_1: u8 = u8::min();
    let u8_2: u8 = 255u8;
    let u8_3: u8 = 2u8;

    let u16_1: u16 = u8_1.into();
    let u16_2: u16 = u8_2.into();
    let u16_3: u16 = u8_3.into();

    assert(u16_1 == 0u16);
    assert(u16_2 == 255u16);
    assert(u16_3 == 2u16);
}

#[test]
fn u16_try_from_u32() {
    let u32_1: u32 = u32::min();
    let u32_2: u32 = u16::max().as_u32();
    let u32_3: u32 = u16::max().as_u32() + 1;
    let u32_4: u32 = 2u32;

    let u16_1 = u16::try_from(u32_1);
    let u16_2 = u16::try_from(u32_2);
    let u16_3 = u16::try_from(u32_3);
    let u16_4 = u16::try_from(u32_4);

    assert(u16_1.is_some());
    assert(u16_1.unwrap() == 0u16);

    assert(u16_2.is_some());
    assert(u16_2.unwrap() == u16::max());

    assert(u16_3.is_none());

    assert(u16_4.is_some());
    assert(u16_4.unwrap() == 2u16);
}

#[test]
fn u16_try_from_u64() {
    let u64_1: u64 = u64::min();
    let u64_2: u64 = 2;
    let u64_3: u64 = u16::max().as_u64();
    let u64_4: u64 = u16::max().as_u64() + 1;

    let u16_1 = u16::try_from(u64_1);
    let u16_2 = u16::try_from(u64_2);
    let u16_3 = u16::try_from(u64_3);
    let u16_4 = u16::try_from(u64_4);

    assert(u16_1.is_some());
    assert(u16_1.unwrap() == 0u16);

    assert(u16_2.is_some());
    assert(u16_2.unwrap() == 2u16);

    assert(u16_3.is_some());
    assert(u16_3.unwrap() == u16::max());

    assert(u16_4.is_none());
}

#[test]
fn u16_try_from_u256() {
    let u256_1: u256 = u256::min();
    let u256_2: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_3: u256 = u16::max().as_u256();
    let u256_4: u256 = 0x1000000000000000000000000000000000000000000000000000000000000000u256;

    let u16_1 = u16::try_from(u256_1);
    let u16_2 = u16::try_from(u256_2);
    let u16_3 = u16::try_from(u256_3);
    let u16_4 = u16::try_from(u256_4);

    assert(u16_1.is_some());
    assert(u16_1.unwrap() == 0u16);

    assert(u16_2.is_some());
    assert(u16_2.unwrap() == 2u16);

    assert(u16_3.is_some());
    assert(u16_3.unwrap() == u16::max());

    assert(u16_4.is_none());
}

#[test]
fn u16_try_from_u128() {
    let u128_1: U128 = U128::new();
    let u128_2: U128 = U128::from((0, 2u64));
    let u128_3: U128 = U128::from((0, u16::max().as_u64()));
    let u128_4: U128 = U128::from((0, u16::max().as_u64() + 1));

    let u16_1 = u16::try_from(u128_1);
    let u16_2 = u16::try_from(u128_2);
    let u16_3 = u16::try_from(u128_3);
    let u16_4 = u16::try_from(u128_4);

    assert(u16_1.is_some());
    assert(u16_1.unwrap() == 0u16);

    assert(u16_2.is_some());
    assert(u16_2.unwrap() == 2u16);

    assert(u16_3.is_some());
    assert(u16_3.unwrap() == u16::max());

    assert(u16_4.is_none());
}
