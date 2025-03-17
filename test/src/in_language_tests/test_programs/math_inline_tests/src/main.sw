library;

mod modulo;
mod divide;
mod add;
mod multiply;
mod subtract;
mod pow;
mod root;
mod log;
mod binary_log;

#[test]
fn math_u8_zero() {
    let my_u8 = u8::zero();
    assert(my_u8.is_zero());

    let other_u8 = 1u8;
    assert(!other_u8.is_zero());
}

#[test]
fn math_u16_zero() {
    let my_u16 = u16::zero();
    assert(my_u16.is_zero());

    let other_u16 = 1u16;
    assert(!other_u16.is_zero());
}

#[test]
fn math_u32_zero() {
    let my_u32 = u32::zero();
    assert(my_u32.is_zero());

    let other_u32 = 1u32;
    assert(!other_u32.is_zero());
}

#[test]
fn math_u64_zero() {
    let my_u64 = u64::zero();
    assert(my_u64.is_zero());

    let other_u64 = 1u64;
    assert(!other_u64.is_zero());
}

#[test]
fn math_u256_zero() {
    let my_u256 = u256::zero();
    assert(my_u256.is_zero());

    let other_u256 = 0x01u256;
    assert(!other_u256.is_zero());
}

#[test]
fn math_b256_zero() {
    let my_b256 = b256::zero();
    assert(my_b256.is_zero());

    let other_b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    assert(!other_b256.is_zero());
}
