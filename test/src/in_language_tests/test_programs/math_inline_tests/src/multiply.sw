library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

// u8
#[test]
fn math_u8_mul() {
    let zero = 0u8;
    let one = 1u8;
    let two = 2u8;
    let four = 4u8;
    let half = u8::max() / 2u8;
    let max = u8::max();

    assert(zero * zero == zero);
    assert(one * zero == zero);
    assert(zero * one == zero);
    assert(zero * half == zero);
    assert(one * one == one);
    assert(one * two == two);
    assert(two * one == two);
    assert(one * half == half);
    assert(two * two == four);
    assert(half * two == max - one);
    assert(max * one == max);
}

#[test(should_revert)]
fn revert_math_u8_overflow_mul() {
    let a = u8::max();
    let b = a * 2;
    log(b);
}

#[test(should_revert)]
fn revert_math_u8_mul_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u8::max();
    let b = a * 2;
    log(b);
}

#[test]
fn math_u8_overflow_mul() {
    let _ = disable_panic_on_overflow();

    let a = (u8::max() / 2) + 1;
    let b = a * 2;

    require(b == 0_u8, b)
}

// u16
#[test]
fn math_u16_mul() {
    let zero = 0u16;
    let one = 1u16;
    let two = 2u16;
    let four = 4u16;
    let half = u16::max() / 2u16;
    let max = u16::max();

    assert(zero * zero == zero);
    assert(one * zero == zero);
    assert(zero * one == zero);
    assert(zero * half == zero);
    assert(one * one == one);
    assert(one * two == two);
    assert(two * one == two);
    assert(one * half == half);
    assert(two * two == four);
    assert(half * two == max - one);
    assert(max * one == max);
}


#[test(should_revert)]
fn revert_math_u16_overflow_mul() {
    let a = u16::max();
    let b = a * 2;
    log(b);
}

#[test(should_revert)]
fn revert_math_u16_mul_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u16::max();
    let b = a * 2;
    log(b);
}

#[test]
fn math_u16_overflow_mul() {
    let _ = disable_panic_on_overflow();

    let a = (u16::max() / 2) + 1;
    let b = a * 2;

    require(b == 0_u16, b)
}

// u32
#[test]
fn math_u32_mul() {
    let zero = 0u32;
    let one = 1u32;
    let two = 2u32;
    let four = 4u32;
    let half = u32::max() / 2u32;
    let max = u32::max();

    assert(zero * zero == zero);
    assert(one * zero == zero);
    assert(zero * one == zero);
    assert(zero * half == zero);
    assert(one * one == one);
    assert(one * two == two);
    assert(two * one == two);
    assert(one * half == half);
    assert(two * two == four);
    assert(half * two == max - one);
    assert(max * one == max);
}

#[test(should_revert)]
fn revert_math_u32_overflow_mul() {
    let a = u32::max();
    let b = a * 2;
    log(b);
}

#[test(should_revert)]
fn revert_math_u32_mul_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u32::max();
    let b = a * 2;
    log(b);
}

#[test]
fn math_u32_overflow_mul() {
    let _ = disable_panic_on_overflow();

    let a = (u32::max() / 2) + 1;
    let b = a * 2;

    require(b == 0_u32, b)
}

// u64
#[test]
fn math_u64_mul() {
    let zero = 0u64;
    let one = 1u64;
    let two = 2u64;
    let four = 4u64;
    let half = u64::max() / 2u64;
    let max = u64::max();

    assert(zero * zero == zero);
    assert(one * zero == zero);
    assert(zero * one == zero);
    assert(zero * half == zero);
    assert(one * one == one);
    assert(one * two == two);
    assert(two * one == two);
    assert(one * half == half);
    assert(two * two == four);
    assert(half * two == max - one);
    assert(max * one == max);
}

// TODO: Uncomment this test https://github.com/FuelLabs/sway/issues/7161 is fixed.
// #[test(should_revert)]
// fn revert_math_u64_overflow_mul() {
//     let a = u64::max();
//     let b = a * 2;
//     log(b);
// }

// TODO: Uncomment this test https://github.com/FuelLabs/sway/issues/7161 is fixed.
// #[test(should_revert)]
// fn revert_math_u64_mul_unsafe_math() {
//     let _ = disable_panic_on_unsafe_math();
//     let a = u64::max();
//     let b = a * 2;
//     log(b);
// }

// TODO: Uncomment this test https://github.com/FuelLabs/sway/issues/7161 is fixed.
// #[test]
// fn math_u64_overflow_mul() {
//     let _ = disable_panic_on_overflow();

//     let a = (u64::max() / 2) + 1;
//     let b = a * 2;

//     require(b == 0_u64, b)
// }

// u256
#[test]
fn math_u256_mul() {
    let zero = u256::zero();
    let one = 0x01u256;
    let two = 0x02u256;
    let four = 0x04u256;
    let half = u256::max() / 0x02u256;
    let max = u256::max();

    assert(zero * zero == zero);
    assert(one * zero == zero);
    assert(zero * one == zero);
    assert(zero * half == zero);
    assert(one * one == one);
    assert(one * two == two);
    assert(two * one == two);
    assert(one * half == half);
    assert(two * two == four);
    assert(half * two == max - one);
    assert(max * one == max);
}

#[test(should_revert)]
fn revert_math_u256_overflow_mul() {
    let a = u256::max();
    let b = a * 2;
    log(b);
}

#[test(should_revert)]
fn revert_math_u256_mul_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u256::max();
    let b = a * 2;
    log(b);
}

#[test]
fn math_u256_overflow_mul() {
    let _ = disable_panic_on_overflow();

    let a = (u256::max() / 2) + 1;
    let b = a * 2;

    require(b == 0.as_u256(), b);
}

