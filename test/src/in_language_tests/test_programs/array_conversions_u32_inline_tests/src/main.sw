library;

use std::array_conversions::u32::*;

#[test]
fn u32_to_le_bytes() {
    let x: u32 = 67305985;
    let result = x.to_le_bytes();

    assert(result[0] == 1_u8);
    assert(result[1] == 2_u8);
    assert(result[2] == 3_u8);
    assert(result[3] == 4_u8);
}

#[test]
fn u32_from_le_bytes() {
    let bytes = [1_u8, 2_u8, 3_u8, 4_u8];
    let result = u32::from_le_bytes(bytes);

    assert(result == 67305985_u32);
}

#[test]
fn u32_to_be_bytes() {
    let x: u32 = 67305985;
    let result = x.to_be_bytes();

    assert(result[0] == 4_u8);
    assert(result[1] == 3_u8);
    assert(result[2] == 2_u8);
    assert(result[3] == 1_u8);
}

#[test]
fn u32_from_be_bytes() {
    let bytes = [4_u8, 3_u8, 2_u8, 1_u8];
    let result = u32::from_be_bytes(bytes);

    assert(result == 67305985_u32);
}
