library;

use std::crypto::scalar::*;

#[test]
fn scalar_new() {
    let new_scalar = Scalar::new();

    assert_eq(new_scalar.bytes().len(), 0);
    assert_eq(new_scalar.bytes().capacity(), 0);
}

#[test]
fn scalar_zero() {
    let zero_scalar = Scalar::zero();

    assert_eq(zero_scalar.bytes().len(), 32);

    assert_eq(b256::try_from(zero_scalar.bytes()).unwrap(), b256::zero());
}

#[test]
fn scalar_is_zero() {
    let zero_scalar = Scalar::zero();
    assert(zero_scalar.is_zero());

    let other_scalar = Scalar::from(b256::zero());
    assert(other_scalar.is_zero());

    let not_zero_scalar = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(!not_zero_scalar.is_zero());
}

#[test]
fn scalar_min() {
    let min_scalar = Scalar::min();

    assert_eq(min_scalar.bytes().len(), 32);
    assert_eq(b256::try_from(min_scalar.bytes()).unwrap(), b256::zero());
}

#[test]
fn scalar_bytes() {
    let zero_scalar = Scalar::zero();

    let zero_bytes = zero_scalar.bytes();
    assert_eq(zero_bytes.len(), 32);
    assert_eq(zero_bytes.capacity(), 32);

    let scalar_1 = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let scalar_1_bytes = scalar_1.bytes();
    assert_eq(scalar_1_bytes.len(), 32);
    assert_eq(scalar_1_bytes.capacity(), 32);

    let scalar_2 = Scalar::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let scalar_2_bytes = scalar_2.bytes();
    assert_eq(scalar_2_bytes.len(), 32);
    assert_eq(scalar_2_bytes.capacity(), 32);
}

#[test]
fn scalar_from_u256() {
    let min = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000000_u256);
    assert_eq(min.bytes().len(), 32);
    assert_eq(min.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(min.bytes())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Scalar::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
    assert_eq(max.bytes().len(), 32);
    assert_eq(max.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(max.bytes())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000000_u256);
    assert_eq(other.bytes().len(), 32);
    assert_eq(other.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(other.bytes())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn scalar_from_b256() {
    let min = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    assert_eq(min.bytes().len(), 32);
    assert_eq(min.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(min.bytes())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Scalar::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert_eq(max.bytes().len(), 32);
    assert_eq(max.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(max.bytes())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Scalar::from(0x1000000000000000000000000000000000000000000000000000000000000000);
    assert_eq(other.bytes().len(), 32);
    assert_eq(other.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(other.bytes())
            .unwrap(),
        0x1000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn scalar_from_u8_array() {
    let min = Scalar::from([
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8,
    ]);
    assert_eq(min.bytes().len(), 32);
    assert_eq(min.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(min.bytes())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Scalar::from([
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
    ]);
    assert_eq(max.bytes().len(), 32);
    assert_eq(max.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(max.bytes())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Scalar::from([
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 1u8,
    ]);
    assert_eq(other.bytes().len(), 32);
    assert_eq(other.bytes().capacity(), 32);
    assert_eq(
        b256::try_from(other.bytes())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn scalar_u256_try_from() {
    let min = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000000_u256);
    let res_min = u256::try_from(min).unwrap();
    assert_eq(
        res_min,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );

    let max = Scalar::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
    let res_max = u256::try_from(max).unwrap();
    assert_eq(
        res_max,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );

    let other = Scalar::from(0x1000000000000000000000000000000000000000000000000000000000000001_u256);
    let other_u256 = u256::try_from(other).unwrap();
    assert_eq(
        other_u256,
        0x1000000000000000000000000000000000000000000000000000000000000001_u256,
    );
}

#[test]
fn scalar_b256_try_from() {
    let min = Scalar::from(0x0000000000000000000000000000000000000000000000000000000000000000_u256);
    let res_min = b256::try_from(min).unwrap();
    assert_eq(
        res_min,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Scalar::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
    let res_max = b256::try_from(max).unwrap();
    assert_eq(
        res_max,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Scalar::from(0x1000000000000000000000000000000000000000000000000000000000000001_u256);
    let other_u256 = b256::try_from(other).unwrap();
    assert_eq(
        other_u256,
        0x1000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn scalar_codec() {
    let scalar = Scalar::new();
    log(scalar);
}
