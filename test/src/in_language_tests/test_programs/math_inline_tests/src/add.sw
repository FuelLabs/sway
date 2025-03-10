library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

#[test]
fn math_u8_add() {
    let zero = u8::zero();
    let one = 1u8;
    let two = 2u8;
    let max = u8::max();
    let half = u8::max() / 2;

    assert(zero + zero == zero);
    assert(zero + one == one);
    assert(one + zero == one);
    assert(one + one == two);
    assert(half + half + one == max);
}

#[test]
fn math_u8_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a = u8::max();
    let b = a + 1;

    require(b == 0_u8, b);

    let c = a + 2;

    require(c == 1_u8, c);

    let d = a + u8::max();

    require(d == u8::max() - 1, d);

    let e = a + (u8::max() - 1);

    require(e == u8::max() - 2, e);
}

#[test(should_revert)]
fn revert_math_u8_overflow_add() {
    let a = u8::max();
    let b = a + 1;
    log(b);
}

#[test(should_revert)]
fn revert_math_u8_add_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u8::max();
    let b = a + 1;
    log(b);
}

#[test]
fn math_u16_add() {
    let zero = u16::zero();
    let one = 1u16;
    let two = 2u16;
    let max = u16::max();
    let half = u16::max() / 2;

    assert(zero + zero == zero);
    assert(zero + one == one);
    assert(one + zero == one);
    assert(one + one == two);
    assert(half + half + one == max);
}

#[test]
fn math_u16_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a: u16 = u16::max();
    let b: u16 = a + 1;

    require(b == 0_u16, b);

    let c = a + 2;

    require(c == 1_u16, c);

    let d = a + u16::max();

    require(d == u16::max() - 1, d);

    let e = a + (u16::max() - 1);

    require(e == u16::max() - 2, e);
}

#[test(should_revert)]
fn revert_math_u16_overflow_add() {
    let a = u16::max();
    let b = a + 1;
    log(b);
}

#[test(should_revert)]
fn revert_math_u16_add_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u16::max();
    let b = a + 1;
    log(b);
}

#[test]
fn math_u32_add() {
    let zero = u32::zero();
    let one = 1u32;
    let two = 2u32;
    let max = u32::max();
    let half = u32::max() / 2;

    assert(zero + zero == zero);
    assert(zero + one == one);
    assert(one + zero == one);
    assert(one + one == two);
    assert(half + half + one == max);
}

#[test]
fn math_u32_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a = u32::max();
    let b = a + 1;

    require(b == 0_u32, b);

    let c = a + 2;

    require(c == 1_u32, c);

    let d = a + u32::max();

    require(d == u32::max() - 1, d);

    let e = a + (u32::max() - 1);

    require(e == u32::max() - 2, e);
}

#[test(should_revert)]
fn revert_math_u32_overflow_add() {
    let a = u32::max();
    let b = a + 1;
    log(b);
}

#[test(should_revert)]
fn revert_math_u32_add_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u32::max();
    let b = a + 1;
    log(b);
}

#[test]
fn math_u64_add() {
    let zero = u64::zero();
    let one = 1u64;
    let two = 2u64;
    let max = u64::max();
    let half = u64::max() / 2;

    assert(zero + zero == zero);
    assert(zero + one == one);
    assert(one + zero == one);
    assert(one + one == two);
    assert(half + half + one == max);
}

#[test]
fn math_u64_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a = u64::max();
    let b = a + 1;

    require(b == 0_u64, b);

    let c = a + 2;

    require(c == 1_u64, c);

    let d = a + u64::max();

    require(d == u64::max() - 1, d);

    let e = a + (u64::max() - 1);

    require(e == u64::max() - 2, e);
}

#[test(should_revert)]
fn revert_math_u64_overflow_add() {
    let a = u64::max();
    let b = a + 1;
    log(b);
}

#[test(should_revert)]
fn revert_math_u64_add_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = u64::max();
    let b = a + 1;
    log(b);
}

#[test]
fn math_u256_add() {
    let zero = u256::zero();
    let one = 0x01u256;
    let two = 0x02u256;
    let max = u256::max();
    let half = u256::max() / 2;

    assert(zero + zero == zero);
    assert(zero + one == one);
    assert(one + zero == one);
    assert(one + one == two);
    assert(half + half + one == max);
}

#[test]
fn math_u256_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a = u256::max();
    let b = a + 1;

    require(b == u256::zero(), b);

    let c = a + 2;

    require(c == 1, c);

    let d = a + u256::max();

    require(d == u256::max() - 1, d);

    let e = a + (u256::max() - 1);

    require(e == u256::max() - 2, e);
}

#[test(should_revert)]
fn revert_math_u256_overflow_add() {
    let a = u256::max();
    let b = a + 1;
    log(b);
}

#[test(should_revert)]
fn revert_math_u256_add_unsafe_math() {
    let a = u256::max();
    let b = a + 1;
    log(b);
}
