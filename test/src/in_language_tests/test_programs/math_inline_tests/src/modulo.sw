library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

// u8
#[test]
pub fn math_u8_modulo() {
    let u8_max = u8::max();

    assert(0u8 % 1u8 == 0u8);
    assert(0u8 % 2u8 == 0u8);
    assert(1u8 % 1u8 == 0u8);
    assert(1u8 % 2u8 == 1u8);

    assert(u8_max % 1u8 == 0u8);
    assert(u8_max % 2u8 == 1u8);
    assert(u8_max % u8_max == 0u8);
    assert(254u8 % u8_max == 254u8);
}

#[test(should_revert)]
pub fn revert_math_u8_modulo_panic_on_undefined_math() {
    log(1u8 % 0u8);
}

#[test(should_revert)]
pub fn revert_math_u8_modulo_panic_on_overflow_disabled() {
    let _ = disable_panic_on_overflow();
    log(1u8 % 0u8);
}

#[test]
pub fn math_u8_modulo_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    assert(0u8 % 0u8 == 0u8);
    assert(1u8 % 0u8 == 0u8);
    assert(u8::max() % 0u8 == 0u8);
}

// u16
#[test]
pub fn math_u16_modulo() {
    let u16_max = u16::max();

    assert(0u16 % 1u16 == 0u16);
    assert(0u16 % 2u16 == 0u16);
    assert(1u16 % 1u16 == 0u16);
    assert(1u16 % 2u16 == 1u16);

    assert(u16_max % 1u16 == 0u16);
    assert(u16_max % 2u16 == 1u16);
    assert(u16_max % u16_max == 0u16);
    assert((u16_max - 1u16) % u16_max == (u16_max - 1u16));
}

#[test(should_revert)]
pub fn revert_math_u16_modulo_panic_on_undefined_math() {
    log(1u16 % 0u16);
}

#[test(should_revert)]
pub fn revert_math_u16_modulo_panic_on_overflow_disabled() {
    let _ = disable_panic_on_overflow();
    log(1u16 % 0u16);
}

#[test]
pub fn math_u16_modulo_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    assert(0u16 % 0u16 == 0u16);
    assert(1u16 % 0u16 == 0u16);
    assert(u16::max() % 0u16 == 0u16);
}

// u32
#[test]
pub fn math_u32_modulo() {
    let u32_max = u32::max();

    assert(0u32 % 1u32 == 0u32);
    assert(0u32 % 2u32 == 0u32);
    assert(1u32 % 1u32 == 0u32);
    assert(1u32 % 2u32 == 1u32);

    assert(u32_max % 1u32 == 0u32);
    assert(u32_max % 2u32 == 1u32);
    assert(u32_max % u32_max == 0u32);
    assert((u32_max - 1u32) % u32_max == (u32_max - 1u32));
}

#[test(should_revert)]
pub fn revert_math_u32_modulo_panic_on_undefined_math() {
    log(1u32 % 0u32);
}

#[test(should_revert)]
pub fn revert_math_u32_modulo_panic_on_overflow_disabled() {
    let _ = disable_panic_on_overflow();
    log(1u32 % 0u32);
}

#[test]
pub fn math_u32_modulo_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    assert(0u32 % 0u32 == 0u32);
    assert(1u32 % 0u32 == 0u32);
    assert(u32::max() % 0u32 == 0u32);
}

// u64
#[test]
pub fn math_u64_modulo() {
    let u64_max = u64::max();

    assert(0u64 % 1u64 == 0u64);
    assert(0u64 % 2u64 == 0u64);
    assert(1u64 % 1u64 == 0u64);
    assert(1u64 % 2u64 == 1u64);

    assert(u64_max % 1u64 == 0u64);
    assert(u64_max % 2u64 == 1u64);
    assert(u64_max % u64_max == 0u64);
    assert((u64_max - 1u64) % u64_max == (u64_max - 1u64));
}

#[test(should_revert)]
pub fn revert_math_u64_modulo_panic_on_undefined_math() {
    log(1u64 % 0u64);
}

#[test(should_revert)]
pub fn revert_math_u64_modulo_panic_on_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    log(1u64 % 0u64);
}

#[test]
pub fn math_u64_modulo_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    assert(0u64 % 0u64 == 0u64);
    assert(1u64 % 0u64 == 0u64);
    assert(u64::max() % 0u64 == 0u64);
}

// u256
#[test]
pub fn math_u256_modulo() {
    let u256_max = u256::max();

    assert(0x0u256 % 0x1u256 == 0x0u256);
    assert(0x1u256 % 0x1u256 == 0x0u256);
    assert(0x1u256 % 0x2u256 == 0x1u256);

    assert(u256_max % 0x1u256 == 0x0u256);
    assert(u256_max % 0x2u256 == 0x1u256);
    assert(u256_max % u256_max == 0x0u256);
    assert((u256_max - 0x1u256) % u256_max == (u256_max - 0x1u256));
}

#[test(should_revert)]
pub fn revert_math_u256_modulo_panic_on_undefined_math() {
    log(0x1u256 % 0x0u256);
}

#[test(should_revert)]
pub fn revert_math_u256_modulo_panic_on_disabled_overflow() {
    let _ = disable_panic_on_overflow();
    log(0x1u256 % 0x0u256);
}

#[test]
pub fn math_u256_modulo_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    assert(0x0u256 % 0x0u256 == 0x0u256);
    assert(0x1u256 % 0x0u256 == 0x0u256);
    assert(u256::max() % 0x0u256 == 0x0u256);
}
