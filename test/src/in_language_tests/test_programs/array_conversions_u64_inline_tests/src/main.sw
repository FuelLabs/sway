library;

use std::array_conversions::u64::*;

#[test]
fn u64_to_le_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_le_bytes();

    assert(result[0] == 1_u8);
    assert(result[1] == 2_u8);
    assert(result[2] == 3_u8);
    assert(result[3] == 4_u8);
    assert(result[4] == 5_u8);
    assert(result[5] == 6_u8);
    assert(result[6] == 7_u8);
    assert(result[7] == 8_u8);
}

#[test]
fn u64_from_le_bytes() {
    let bytes = [1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8];
    let result = u64::from_le_bytes(bytes);

    assert(result == 578437695752307201);
}

#[test]
fn u64_to_be_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_be_bytes();

    assert(result[0] == 8_u8);
    assert(result[1] == 7_u8);
    assert(result[2] == 6_u8);
    assert(result[3] == 5_u8);
    assert(result[4] == 4_u8);
    assert(result[5] == 3_u8);
    assert(result[6] == 2_u8);
    assert(result[7] == 1_u8);
}

#[test]
fn u64_from_be_bytes() {
    let bytes = [8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8, 2_u8, 1_u8];
    let result = u64::from_be_bytes(bytes);

    assert(result == 578437695752307201);
}
