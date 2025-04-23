//! Functions to assert a given condition.
library;

use ::logging::log;
use ::revert::revert;
use ::error_signals::{FAILED_ASSERT_EQ_SIGNAL, FAILED_ASSERT_NE_SIGNAL, FAILED_ASSERT_SIGNAL};
use ::codec::AbiEncode;
use ::ops::*;
use ::never::*;

/// Asserts that the given `condition` will always be `true` during runtime.
///
/// # Additional Information
///
/// To check for conditions that may not be `true`, use `std::revert::require` instead.
/// For more information, see the Wiki article on [Assertion](https://en.wikipedia.org/wiki/Assertion_(software_development)#Comparison_with_error_handling).
///
/// # Arguments
///
/// * `condition`: [bool] - The condition which will be asserted to be `true`.
///
/// # Reverts
///
/// * Reverts when `condition` is `false`.
///
/// # Examples
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
        revert(FAILED_ASSERT_SIGNAL);
    }
}

/// Asserts that the given values `v1` & `v2` will always be equal during runtime.
///
/// # Arguments
///
/// * `v1`: [T] - The first value to compare.
/// * `v2`: [T] - The second value to compare.
///
/// # Reverts
///
/// * Reverts when `v1` != `v2`.
///
/// # Examples
///
/// ```sway
/// fn foo(a: u64, b: u64) {
///     assert_eq(a, b);
///     // if code execution continues, that means `a` is equal to `b`
///     log("a is equal to b");
/// }
/// ```
#[cfg(experimental_new_encoding = false)]
pub fn assert_eq<T>(v1: T, v2: T)
where
    T: PartialEq,
{
    if (v1 != v2) {
        log(v1);
        log(v2);
        revert(FAILED_ASSERT_EQ_SIGNAL);
    }
}

#[cfg(experimental_new_encoding = true)]
pub fn assert_eq<T>(v1: T, v2: T)
where
    T: PartialEq + AbiEncode,
{
    if (v1 != v2) {
        log(v1);
        log(v2);
        revert(FAILED_ASSERT_EQ_SIGNAL);
    }
}

/// Asserts that the given values `v1` & `v2` will never be equal during runtime.
///
/// # Arguments
///
/// * `v1`: [T] - The first value to compare.
/// * `v2`: [T] - The second value to compare.
///
/// # Reverts
///
/// * Reverts when `v1` == `v2`.
///
/// # Examples
///
/// ```sway
/// fn foo(a: u64, b: u64) {
///     assert_ne(a, b);
///     // if code execution continues, that means `a` is not equal to `b`
///     log("a is not equal to b");
/// }
/// ```
#[cfg(experimental_new_encoding = false)]
pub fn assert_ne<T>(v1: T, v2: T)
where
    T: PartialEq,
{
    if (v1 == v2) {
        log(v1);
        log(v2);
        revert(FAILED_ASSERT_NE_SIGNAL);
    }
}

#[cfg(experimental_new_encoding = true)]
pub fn assert_ne<T>(v1: T, v2: T)
where
    T: PartialEq + AbiEncode,
{
    if (v1 == v2) {
        log(v1);
        log(v2);
        revert(FAILED_ASSERT_NE_SIGNAL);
    }
}
