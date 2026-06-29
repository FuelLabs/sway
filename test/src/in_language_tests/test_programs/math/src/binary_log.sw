library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

// u8
#[test]
fn math_log2_u8() {
    let max_u8 = u8::max();

    assert_eq(2u8.log2(), 1u8);
    assert_eq(8u8.log2(), 3u8);
    assert_eq(100u8.log2(), 6u8);
    assert_eq(max_u8.log2(), 7u8);
}

#[test(should_revert)]
fn math_u8_log2_fail_x_0() {
    let result = 0_u8.log2();
    log(result);
}

#[test(should_revert)]
fn math_u8_log2_fail_x_0_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let result = 0_u8.log2();
    log(result);
}

#[test]
fn math_u8_log2_fail_x_0_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 0_u8.log2();
    assert_eq(result, 0u8);
}

// u16
#[test]
fn math_log2_u16() {
    let max_u16 = u16::max();
    assert_eq(2u16.log2(), 1u16);
    assert_eq(8u16.log2(), 3u16);
    assert_eq(100u16.log2(), 6u16);
    assert_eq(1025u16.log2(), 10u16);
    assert_eq(max_u16.log2(), 15u16);
}

#[test(should_revert)]
fn math_u16_log2_fail_x_0() {
    let result = 0_u16.log2();
    log(result);
}

#[test(should_revert)]
fn math_u16_log2_fail_x_0_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let result = 0_u16.log2();
    log(result);
}

#[test]
fn math_u16_log2_fail_x_0_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 0_u16.log2();
    assert_eq(result, 0u16);
}

// u32
#[test]
fn math_log2_u32() {
    let max_u32 = u32::max();
    assert_eq(2u32.log2(), 1u32);
    assert_eq(8u32.log2(), 3u32);
    assert_eq(100u32.log2(), 6u32);
    assert_eq(1025u32.log2(), 10u32);
    assert_eq(max_u32.log2(), 31u32);
}

#[test(should_revert)]
fn math_u32_log2_fail_x_0() {
    let result = 0_u32.log2();
    log(result);
}

#[test(should_revert)]
fn math_u32_log2_fail_x_0_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let result = 0_u32.log2();
    log(result);
}

#[test]
fn math_u32_log2_fail_x_0_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 0_u32.log2();
    assert_eq(result, 0u32);
}

// u64
#[test]
fn math_log2_u64() {
    let max_u64 = u64::max();
    assert_eq(2u64.log2(), 1u64);
    assert_eq(8u64.log2(), 3u64);
    assert_eq(100u64.log2(), 6u64);
    assert_eq(1025u64.log2(), 10u64);
    assert_eq(max_u64.log2(), 63u64);
}

#[test(should_revert)]
fn math_u64_log2_fail_x_0() {
    let result = 0_u64.log2();
    log(result);
}

#[test(should_revert)]
fn math_u64_log2_fail_x_0_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let result = 0_u64.log2();
    log(result);
}

#[test]
fn math_u64_log2_fail_x_0_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 0_u64.log2();
    assert_eq(result, 0u64);
}

// u256
#[test]
fn math_log2_u256() {
    let max_u256 = u256::max();
    assert_eq(0x2u256.log2(), 0x1u256);
    assert_eq(0x401u256.log2(), 0xau256);
    assert_eq(max_u256.log2(), 0xffu256);
    assert_eq(0x8u256.log2(), 0x3u256);
    assert_eq(0x64u256.log2(), 0x6u256);
}

#[test(should_revert)]
fn math_u256_log2_fail_x_0() {
    let result = u256::from(0_u64).log2();
    log(result);
}

#[test(should_revert)]
fn math_u256_log2_fail_x_0_disable_overflow() {
    let _ = disable_panic_on_overflow();
    let result = u256::from(0_u64).log2();
    log(result);
}

#[test]
fn math_u256_log2_fail_x_0_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = u256::from(0_u64).log2();
    assert_eq(result, 0x00u256);
}
