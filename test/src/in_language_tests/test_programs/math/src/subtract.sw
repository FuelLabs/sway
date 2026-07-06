library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

// u8
#[test]
fn math_u8_sub() {
    assert_eq((u8::max() - u8::max()), 0u8);
    assert_eq((u8::min() - u8::min()), 0u8);
    assert_eq((10u8 - 5u8), 5u8);

    let zero = 0u8;
    let one = 1u8;
    let two = 2u8;
    let three = 3u8;

    assert_eq(zero - zero, zero);
    assert_eq(one - zero, one);
    assert_eq(two - zero, two);
    assert_eq(one - one, zero);
    assert_eq(two - one, one);
    assert_eq(three - one, two);
    assert_eq(three - two, one);
    assert_eq(two - two, zero);
    assert_eq(three - three, zero);
}

#[test(should_revert)]
fn revert_math_u8_underflow_sub() {
    let a = 0u8;
    let b = 1u8;
    let c = a - b;
    log(c);
}

#[test(should_revert)]
fn revert_math_u8_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = 0u8;
    let b = 1u8;
    let c = a - b;
    log(c);
}

#[test]
fn math_u8_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = 0u8;
    let b = 1u8;

    let c = a - b;
    assert_eq(c, u8::max());

    let d = u8::max();

    let e = a - d;
    assert_eq(e, b);
}

// u16
#[test]
fn math_u16_sub() {
    assert_eq((u16::max() - u16::max()), 0u16);
    assert_eq((u16::min() - u16::min()), 0u16);
    assert_eq((10u16 - 5u16), 5u16);

    let zero = 0u16;
    let one = 1u16;
    let two = 2u16;
    let three = 3u16;

    assert_eq(zero - zero, zero);
    assert_eq(one - zero, one);
    assert_eq(two - zero, two);
    assert_eq(one - one, zero);
    assert_eq(two - one, one);
    assert_eq(three - one, two);
    assert_eq(three - two, one);
    assert_eq(two - two, zero);
    assert_eq(three - three, zero);
}

#[test(should_revert)]
fn revert_math_u16_underflow_sub() {
    let a = 0u16;
    let b = 1u16;
    let c = a - b;
    log(c);
}

#[test(should_revert)]
fn revert_math_u16_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = 0u16;
    let b = 1u16;
    let c = a - b;
    log(c);
}

#[test]
fn math_u16_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = 0u16;
    let b = 1u16;

    let c = a - b;
    assert_eq(c, u16::max());

    let d = u16::max();

    let e = a - d;
    assert_eq(e, b);
}

// u32
#[test]
fn math_u32_sub() {
    assert_eq((u32::max() - u32::max()), 0u32);
    assert_eq((u32::min() - u32::min()), 0u32);
    assert_eq((10u32 - 5u32), 5u32);

    let zero = 0u32;
    let one = 1u32;
    let two = 2u32;
    let three = 3u32;

    assert_eq(zero - zero, zero);
    assert_eq(one - zero, one);
    assert_eq(two - zero, two);
    assert_eq(one - one, zero);
    assert_eq(two - one, one);
    assert_eq(three - one, two);
    assert_eq(three - two, one);
    assert_eq(two - two, zero);
    assert_eq(three - three, zero);
}

#[test(should_revert)]
fn revert_math_u32_underflow_sub() {
    let a = 0u32;
    let b = 1u32;
    let c = a - b;
    log(c);
}

#[test(should_revert)]
fn revert_math_u32_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = 0u32;
    let b = 1u32;
    let c = a - b;
    log(c);
}

#[test]
fn math_u32_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = 0u32;
    let b = 1u32;

    let c = a - b;
    assert_eq(c, u32::max());

    let d = u32::max();

    let e = a - d;
    assert_eq(e, b);
}

// u64
#[test]
fn math_u64_sub() {
    assert_eq((u64::max() - u64::max()), 0u64);
    assert_eq((u64::min() - u64::min()), 0u64);
    assert_eq((10u64 - 5u64), 5u64);

    let zero = 0u64;
    let one = 1u64;
    let two = 2u64;
    let three = 3u64;

    assert_eq(zero - zero, zero);
    assert_eq(one - zero, one);
    assert_eq(two - zero, two);
    assert_eq(one - one, zero);
    assert_eq(two - one, one);
    assert_eq(three - one, two);
    assert_eq(three - two, one);
    assert_eq(two - two, zero);
    assert_eq(three - three, zero);
}

#[test(should_revert)]
fn revert_math_u64_underflow_sub() {
    let a = 0u64;
    let b = 1u64;
    let c = a - b;
    log(c);
}

#[test(should_revert)]
fn revert_math_u64_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = 0u64;
    let b = 1u64;
    let c = a - b;
    log(c);
}

#[test]
fn math_u64_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = 0u64;
    let b = 1u64;

    let c = a - b;
    assert_eq(c, u64::max());

    let d = u64::max();

    let e = a - d;
    assert_eq(e, b);
}

// u256
#[test]
fn math_u256_sub() {
    assert_eq((u256::max() - u256::max()), u256::zero());
    assert_eq((u256::min() - u256::min()), u256::zero());
    assert_eq((0x0au256 - 0x05u256), 0x05u256);

    let zero = 0x00u256;
    let one = 0x01u256;
    let two = 0x02u256;
    let three = 0x03u256;

    assert_eq(zero - zero, zero);
    assert_eq(one - zero, one);
    assert_eq(two - zero, two);
    assert_eq(one - one, zero);
    assert_eq(two - one, one);
    assert_eq(three - one, two);
    assert_eq(three - two, one);
    assert_eq(two - two, zero);
    assert_eq(three - three, zero);
}

#[test(should_revert)]
fn revert_math_u256_underflow_sub() {
    let a = 0x00u256;
    let b = 0x01u256;
    let c = a - b;
    log(c);
}

#[test(should_revert)]
fn revert_math_u256_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = 0x00u256;
    let b = 0x01u256;
    let c = a - b;
    log(c);
}

#[test]
fn math_u256_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = 0x00u256;
    let b = 0x01u256;

    let c = a - b;
    assert_eq(c, u256::max());

    let d = u256::max();

    let e = a - d;
    assert_eq(e, b);
}
