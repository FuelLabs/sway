library;

use std::{b512::B512, bytes::Bytes, primitive_conversions::b256::*, u128::U128};

#[test]
fn b256_try_from_bytes() {
    let mut initial_bytes = Bytes::with_capacity(32);
    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        initial_bytes.push(51u8);
        i += 1;
    }
    let res1 = b256::try_from(initial_bytes);
    let expected1 = 0x3333333333333333333333333333333333333333333333333333333333333333;
    assert(res1.is_some());
    assert(res1.unwrap() == expected1);

    let mut second_bytes = Bytes::with_capacity(33);
    i = 0;
    while i < 33 {
        // 0x33 is 51 in decimal
        second_bytes.push(51u8);
        i += 1;
    }
    let res2 = b256::try_from(second_bytes);
    assert(res2.is_none());

    // bytes is still available to use:
    assert(second_bytes.len() == 33);
    assert(second_bytes.capacity() == 33);

    let mut third_bytes = Bytes::with_capacity(31);
    let mut i = 0;
    while i < 31 {
        // 0x33 is 51 in decimal
        third_bytes.push(51u8);
        i += 1;
    }
    let res3 = b256::try_from(third_bytes);
    assert(res3.is_none());
}

#[test]
fn b256_try_from_b512() {
    let b512_value = B512::new();
    let b256_value = b256::try_from(b512_value);
    assert(b256_value.is_some());

    let b512_value = B512::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        b256::zero(),
    ));
    let b256_value = b256::try_from(b512_value);
    assert(b256_value.is_none());
}

#[test]
fn b256_from_u256() {
    let u256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let u256_2 = 0x0000000000000000000000000000000000000000000000000000000000000001_u256;
    let u256_3 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256;

    let res1 = b256::from(u256_1);
    let res2 = b256::from(u256_2);
    let res3 = b256::from(u256_3);

    assert(res1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(res2 == 0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(res3 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}

#[test]
fn b256_into_u256() {
    let u256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let u256_2 = 0x0000000000000000000000000000000000000000000000000000000000000001_u256;
    let u256_3 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256;

    let res1: b256 = u256_1.into();
    let res2: b256 = u256_2.into();
    let res3: b256 = u256_3.into();

    assert(res1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(res2 == 0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(res3 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}

#[test]
fn b256_from_u128() {
    let b256_value1 = b256::from(U128::from((u64::min(), u64::min())));
    let b256_value2 = b256::from(U128::from((1_u64, 1_u64)));
    let b256_value3 = b256::from(U128::from((u64::max(), u64::max())));
    let b256_value4 = b256::from(U128::from((u64::max(), u64::min())));
    let b256_value5 = b256::from(U128::from((u64::min(), u64::max())));

    assert(
        b256_value1 == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(
        b256_value2 == 0x0000000000000000000000000000000000000000000000010000000000000001,
    );
    assert(
        b256_value3 == 0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert(
        b256_value4 == 0x00000000000000000000000000000000ffffffffffffffff0000000000000000,
    );
    assert(
        b256_value5 == 0x000000000000000000000000000000000000000000000000ffffffffffffffff,
    );
}

#[test]
fn b256_into_u128() {
    let u128_1 = U128::from((u64::min(), u64::min()));
    let u128_2 = U128::from((1_u64, 1_u64));
    let u128_3 = U128::from((u64::max(), u64::max()));
    let u128_4 = U128::from((u64::max(), u64::min()));
    let u128_5 = U128::from((u64::min(), u64::max()));

    let b256_value1: b256 = u128_1.into();
    let b256_value2: b256 = u128_2.into();
    let b256_value3: b256 = u128_3.into();
    let b256_value4: b256 = u128_4.into();
    let b256_value5: b256 = u128_5.into();

    assert(
        b256_value1 == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(
        b256_value2 == 0x0000000000000000000000000000000000000000000000010000000000000001,
    );
    assert(
        b256_value3 == 0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert(
        b256_value4 == 0x00000000000000000000000000000000ffffffffffffffff0000000000000000,
    );
    assert(
        b256_value5 == 0x000000000000000000000000000000000000000000000000ffffffffffffffff,
    );
}

#[test]
fn b256_from_tuple() {
    let b256_1 = b256::from((0, 0, 0, 0));
    let b256_2 = b256::from((1, 2, 3, 4));
    let b256_3 = b256::from((u64::max(), u64::max(), u64::max(), u64::max()));

    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(b256_2 == 0x0000000000000001000000000000000200000000000000030000000000000004);
    assert(b256_3 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}

#[test]
fn b256_into_tuple() {
    let tuple_1 = (0, 0, 0, 0);
    let tuple_2 = (1, 2, 3, 4);
    let tuple_3 = (u64::max(), u64::max(), u64::max(), u64::max());

    let b256_1: b256 = tuple_1.into();
    let b256_2: b256 = tuple_2.into();
    let b256_3: b256 = tuple_3.into();

    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(b256_2 == 0x0000000000000001000000000000000200000000000000030000000000000004);
    assert(b256_3 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}
