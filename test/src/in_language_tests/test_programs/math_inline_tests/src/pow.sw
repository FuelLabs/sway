library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

// u8
#[test]
fn math_power_u8() {
    assert(2u8.pow(2u32) == 4u8);
    assert(2u8 ** 2u32 == 4u8);

    assert(2u8.pow(3u32) == 8u8);
    assert(2u8 ** 3u32 == 8u8);

    assert(4u8.pow(3u32) == 64u8);
    assert(4u8 ** 3u32 == 64u8);

    assert(3u8.pow(4u32) == 81u8);
    assert(3u8 ** 4u32 == 81u8);

    assert(10u8.pow(2u32) == 100u8);
    assert(10u8 ** 2u32 == 100u8);

    assert(5u8.pow(3u32) == 125u8);
    assert(5u8 ** 3u32 == 125u8);

    assert(3u8.pow(5u32) == 243u8);
    assert(3u8 ** 5u32 == 243u8);

    assert(2u8.pow(0u32) == 1u8);
    assert(2u8 ** 0u32 == 1u8);

    assert(0u8.pow(1u32) == 0u8);
    assert(0u8 ** 1u32 == 0u8);

    assert(0u8.pow(2u32) == 0u8);
    assert(0u8 ** 2u32 == 0u8);
}

#[test(should_revert)]
fn revert_math_u8_pow_overflow() {
    let result = 2_u8.pow(8);
    log(result);
}

#[test(should_revert)]
fn revert_math_u8_pow_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 2_u8.pow(8);
    log(result);
}

#[test]
fn math_u8_overflow_pow() {
    let _ = disable_panic_on_overflow();

    let a = u8::max();
    let b = a.pow(2);

    require(b == 0_u8, b);
}

// u16
#[test]
fn math_power_u16() {
    assert(2u16.pow(2u32) == 4u16);
    assert(2u16 ** 2u32 == 4u16);

    assert(2u16.pow(3u32) == 8u16);
    assert(2u16 ** 3u32 == 8u16);

    assert(42u16.pow(2u32) == 1764u16);
    assert(42u16 ** 2u32 == 1764u16);

    assert(20u16.pow(3u32) == 8000u16);
    assert(20u16 ** 3u32 == 8000u16);

    assert(15u16.pow(4u32) == 50625u16);
    assert(15u16 ** 4u32 == 50625u16);

    assert(2u16.pow(0u32) == 1u16);
    assert(2u16 ** 0u32 == 1u16);

    assert(0u16.pow(1u32) == 0u16);
    assert(0u16 ** 1u32 == 0u16);

    assert(0u16.pow(2u32) == 0u16);
    assert(0u16 ** 2u32 == 0u16);
}

#[test(should_revert)]
fn revert_math_u16_pow_overflow() {
    let result = 2_u16.pow(16);
    log(result);
}

#[test(should_revert)]
fn revert_math_u16_pow_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 2_u16.pow(16);
    log(result);
}

#[test]
fn math_u16_overflow_pow() {
    let _ = disable_panic_on_overflow();

    let a = u16::max();
    let b = a.pow(2);

    require(b == 0_u16, b);
}

// u32
#[test]
fn math_power_u32() {
    assert(2u32.pow(2u32) == 4u32);
    assert(2u32 ** 2u32 == 4u32);

    assert(2u32.pow(3u32) == 8u32);
    assert(2u32 ** 3u32 == 8u32);

    assert(42u32.pow(2u32) == 1764u32);
    assert(42u32 ** 2u32 == 1764u32);

    assert(100u32.pow(4u32) == 100000000u32);
    assert(100u32 ** 4u32 == 100000000u32);

    assert(2u32.pow(0u32) == 1u32);
    assert(2u32 ** 0u32 == 1u32);

    assert(0u32.pow(1u32) == 0u32);
    assert(0u32 ** 1u32 == 0u32);

    assert(0u32.pow(2u32) == 0u32);
    assert(0u32 ** 2u32 == 0u32);
}

#[test(should_revert)]
fn revert_math_u32_pow_overflow() {
    let result = 2_u32.pow(32);
    log(result);
}

#[test(should_revert)]
fn revert_math_u32_pow_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 2_u32.pow(32);
    log(result);
}

#[test]
fn math_u32_overflow_pow() {
    let _ = disable_panic_on_overflow();

    let a = u32::max();
    let b = a.pow(2);

    require(b == 0_u32, b);
}

// u64
#[test]
fn math_power_u64() {
    assert(2.pow(2) == 4);
    assert(2 ** 2 == 4);

    assert(2.pow(3) == 8);
    assert(2 ** 3 == 8);

    assert(42.pow(2) == 1764);
    assert(42 ** 2 == 1764);

    assert(42.pow(3) == 74088);
    assert(42 ** 3 == 74088);

    assert(100.pow(5) == 10000000000);
    assert(100 ** 5 == 10000000000);

    assert(100.pow(8) == 10000000000000000);
    assert(100 ** 8 == 10000000000000000);

    assert(100.pow(9) == 1000000000000000000);
    assert(100 ** 9 == 1000000000000000000);

    assert(2.pow(0) == 1);
    assert(2 ** 0 == 1);

    assert(0.pow(1) == 0);
    assert(0 ** 1 == 0);

    assert(0.pow(2) == 0);
    assert(0 ** 2 == 0);
}

#[test(should_revert)]
fn revert_math_u64_pow_overflow() {
    let result = 2_u64.pow(64);
    log(result);
}

#[test(should_revert)]
fn revert_math_u64_pow_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 2_u64.pow(64);
    log(result);
}

#[test]
fn math_u64_overflow_pow() {
    let _ = disable_panic_on_overflow();

    let a = u64::max();
    let b = a.pow(2);

    require(b == 0_u64, b);
}

// u256
#[test]
fn math_power_u256() {
    let zero = 0x00u256;
    let one = 0x01u256;
    let two = 0x02u256;
    let three = 0x03u256;
    let four = 0x04u256;
    let five = 0x05u256;
    let eight = 0x08u256;
    let nine = 0x09u256;

    // 5^2 = 25 = 0x19
    assert_eq(
        five
            .pow(2),
        0x0000000000000000000000000000000000000000000000000000000000000019u256,
    );

    // 5^28 = 0x204FCE5E3E2502611 (see https://www.wolframalpha.com/input?i=convert+5%5E28+in+hex)
    assert_eq(five.pow(28), 0x0000000000000000204FCE5E3E2502611u256);

    assert(two.pow(2) == four);
    assert(two ** 2 == four);

    assert(two.pow(3) == eight);
    assert(two ** 3 == eight);

    assert(three.pow(2) == nine);
    assert(three ** 2 == nine);

    assert(0x2au256.pow(2) == 0x06e4u256);
    assert(0x2au256 ** 2 == 0x06e4u256);

    assert(0x2au256.pow(3) == 0x012168u256);
    assert(0x2au256 ** 3 == 0x012168u256);

    assert(0x64u256.pow(5) == 0x2540be400u256);
    assert(0x64u256 ** 5 == 0x2540be400u256);

    assert(0x64u256.pow(8) == 0x2386f26fc10000u256);
    assert(0x64u256 ** 8 == 0x2386f26fc10000u256);

    assert(0x64u256.pow(9) == 0xde0b6b3a7640000u256);
    assert(0x64u256 ** 9 == 0xde0b6b3a7640000u256);

    assert(two.pow(0) == one);
    assert(two ** 0 == one);

    assert(zero.pow(1) == zero);
    assert(zero ** 1 == zero);

    assert(zero.pow(2) == zero);
    assert(zero ** 2 == zero);
}

#[test(should_revert)]
fn revert_math_u256_pow_overflow() {
    let result = 2.as_u256().pow(256);
    log(result);
}

#[test(should_revert)]
fn revert_math_u256_pow_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let result = 2.as_u256().pow(256);
    log(result);
}

#[test]
fn math_u256_overflow_pow() {
    let _ = disable_panic_on_overflow();

    let a = u256::max();
    let b = a.pow(2);

    require(b == 0.as_u256(), b);
}
