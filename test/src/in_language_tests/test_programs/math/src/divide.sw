library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

// u8
#[test]
fn math_u8_divide() {
    let zero = 0u8;
    let one = 1u8;
    let two = 2u8;
    let max = u8::max();

    assert_eq(zero / one, zero);
    assert_eq(zero / max, zero);
    assert_eq(one / one, one);
    assert_eq(two / one, two);
    assert_eq(one / two, zero);
    assert_eq(two / two, one);
    assert_eq(max / two, 127u8);
    assert_eq(max / one, max);
}

#[test]
fn math_u8_divide_by_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let zero = 0u8;
    let one = 1u8;
    let max = u8::max();

    let result = one / zero;
    assert_eq(result, zero);

    let res_2 = max / zero;
    assert_eq(res_2, zero);
}

#[test(should_revert)]
fn revert_math_u8_divide_by_zero() {
    let zero = 0u8;
    let one = 1u8;

    let result = one / zero;
    log(result);
}

#[test(should_revert)]
fn revert_math_u8_divide_by_zero_with_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let zero = 0u8;
    let one = 1u8;

    let result = one / zero;
    assert_eq(result, zero);
}

// u16
#[test]
fn math_u16_divide() {
    let zero = 0u16;
    let one = 1u16;
    let two = 2u16;
    let max = u16::max();

    assert_eq(zero / one, zero);
    assert_eq(zero / max, zero);
    assert_eq(one / one, one);
    assert_eq(two / one, two);
    assert_eq(one / two, zero);
    assert_eq(two / two, one);
    assert_eq(max / two, 32767u16);
    assert_eq(max / one, max);
}

#[test]
fn math_u16_divide_by_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let zero = 0u16;
    let one = 1u16;
    let max = u16::max();

    let result = one / zero;
    assert_eq(result, zero);

    let res_2 = max / zero;
    assert_eq(res_2, zero);
}

#[test(should_revert)]
fn revert_math_u16_divide_by_zero() {
    let zero = 0u16;
    let one = 1u16;

    let result = one / zero;
    log(result);
}

#[test(should_revert)]
fn revert_math_u16_divide_by_zero_with_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let zero = 0u16;
    let one = 1u16;

    let result = one / zero;
    assert_eq(result, zero);
}

// u32
#[test]
fn math_u32_divide() {
    let zero = 0u32;
    let one = 1u32;
    let two = 2u32;
    let max = u32::max();

    assert_eq(zero / one, zero);
    assert_eq(zero / max, zero);
    assert_eq(one / one, one);
    assert_eq(two / one, two);
    assert_eq(one / two, zero);
    assert_eq(two / two, one);
    assert_eq(max / two, 2147483647u32);
    assert_eq(max / one, max);
}

#[test]
fn math_u32_divide_by_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let zero = 0u32;
    let one = 1u32;
    let max = u32::max();

    let result = one / zero;
    assert_eq(result, zero);

    let res_2 = max / zero;
    assert_eq(res_2, zero);
}

#[test(should_revert)]
fn revert_math_u32_divide_by_zero() {
    let zero = 0u32;
    let one = 1u32;

    let result = one / zero;
    log(result);
}

#[test(should_revert)]
fn revert_math_u32_divide_by_zero_with_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let zero = 0u32;
    let one = 1u32;

    let result = one / zero;
    assert_eq(result, zero);
}

// u64
#[test]
fn math_u64_divide() {
    let zero = 0u64;
    let one = 1u64;
    let two = 2u64;
    let max = u64::max();

    assert_eq(zero / one, zero);
    assert_eq(zero / max, zero);
    assert_eq(one / one, one);
    assert_eq(two / one, two);
    assert_eq(one / two, zero);
    assert_eq(two / two, one);
    assert_eq(max / two, 9223372036854775807u64);
    assert_eq(max / one, max);
}

#[test]
fn math_u64_divide_by_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let zero = 0u64;
    let one = 1u64;
    let max = u64::max();

    let result = one / zero;
    assert_eq(result, zero);

    let res_2 = max / zero;
    assert_eq(res_2, zero);
}
#[test(should_revert)]
fn revert_math_u64_divide_by_zero() {
    let zero = 0u64;
    let one = 1u64;

    let result = one / zero;
    log(result);
}

#[test(should_revert)]
fn revert_math_u64_divide_by_zero_with_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let zero = 0u64;
    let one = 1u64;

    let result = one / zero;
    assert_eq(result, zero);
}

// u256
#[test]
fn math_u256_divide() {
    let zero = u256::zero();
    let one = 0x01u256;
    let two = 0x02u256;
    let max = u256::max();

    assert_eq(zero / one, zero);
    assert_eq(zero / max, zero);
    assert_eq(one / one, one);
    assert_eq(two / one, two);
    assert_eq(one / two, zero);
    assert_eq(two / two, one);
    assert_eq(
        max / two,
        0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256,
    );
    assert_eq(max / one, max);
}

#[test]
fn math_u256_divide_by_zero_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let zero = u256::zero();
    let one = 0x01u256;
    let max = u256::max();

    let result = one / zero;
    assert_eq(result, zero);

    let res_2 = max / zero;
    assert_eq(res_2, zero);
}

#[test(should_revert)]
fn revert_math_u256_divide_by_zero() {
    let zero = u256::zero();
    let one = 0x01u256;

    let result = one / zero;
    log(result);
}

#[test(should_revert)]
fn revert_math_u256_divide_by_zero_with_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    let zero = u256::zero();
    let one = 0x01u256;

    let result = one / zero;
    assert_eq(result, zero);
}
