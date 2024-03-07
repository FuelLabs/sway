library;

pub fn convert_to_u256() {
    // Convert any unsigned integer to `u256`
    // ANCHOR: to_u256
    let u8_1: u8 = 2u8;
    let u16_1: u16 = 2u16;
    let u32_1: u32 = 2u32;
    let u64_1: u64 = 2u64;
    let b256_1: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let u256_from_u8: u256 = u8_1.as_u256();
    let u256_from_u16: u256 = u16_1.as_u256();
    let u256_from_u32: u256 = u32_1.as_u256();
    let u256_from_u64: u256 = u64_1.as_u256();
    let u256_from_b256: u256 = b256_1.as_u256();
    // ANCHOR_END: to_u256
}
