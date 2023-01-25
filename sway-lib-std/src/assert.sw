library assert;

use core::ops::Eq;
use ::logging::log;
use ::revert::revert;

/// Asserts that the given `condition` will always be `true` during runtime.
/// To check for conditions that may not be `true`, use `std::revert::require` instead.
/// For more information, see the Wiki article on [Assertion](https://en.wikipedia.org/wiki/Assertion_(software_development)#Comparison_with_error_handling).
///
/// ### Arguments
///
/// * `condition` - The condition which will be asserted to be `true`.
///
/// ### Reverts
///
/// Reverts when `condition` is `false`.
///
/// ### Examples
///
/// ```sway
/// fn foo(a: u64, b: u64) {
///     assert(a == b);
///     // if code execution continues, that means a was equal to b
///     log("a is equal to b");
/// }
/// ```
pub fn assert(condition: bool) {
    if !condition {
        revert(0);
    }
}

/// Asserts that the given values `v1` & `v2` will always be equal during runtime.
///
/// ### Arguments
///
/// * `v1` - The first value to compare.
/// * `v2` - The second value to compare.
///
/// ### Reverts
///
/// Reverts when `v1` != `v1`.
///
/// ### Examples
///
/// ```sway
/// fn foo(a: u64, b: u64) {
///     assert_eq(a, b);
///     // if code execution continues, that means a was equal to b
///     log("a is equal to b");
/// }
/// ```
fn assert_eq<T>(v1: T, v2: T) where T: Eq {
    if (v1 != v2) {
        log(v1);
        log(v2);
        revert(0xffff_ffff_ffff_0004);
    }
}

#[test()]
fn test_assert_eq_u64() {
    let a = 42;
    let b = 40 + 2;
    assert_eq(a, b);
}

#[test()]
fn test_assert_eq_u32() {
    let a = 42u32;
    let b = 40u32 + 2u32;
    assert_eq(a, b);
}

#[test()]
fn test_assert_eq_u16() {
    let a = 42u16;
    let b = 40u16 + 2u16;
    assert_eq(a, b);
}

#[test()]
fn test_assert_eq_u8() {
    let a = 42u8;
    let b = 40u8 + 2u8;
    assert_eq(a, b);
}

#[test()]
fn test_assert_eq_b256() {
    let a: b256 = 0b0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000001_0000000000000000000000000000000000000000000000000000000000000010;
    let b: b256 = 0b1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000000_1000000000000000000000000000000000000000000000000000000000000001 << 1;
    assert_eq(a, b);
}
