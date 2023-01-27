library assert;

use ::logging::log;
use ::revert::revert;
use ::error_signals::FAILED_ASSERT_EQ_SIGNAL;


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
/// Reverts when `v1` != `v2`.
///
/// ### Examples
///
/// ```sway
/// fn foo(a: u64, b: u64) {
///     assert_eq(a, b);
///     // if code execution continues, that means `a` is equal to `b`
///     log("a is equal to b");
/// }
/// ```
pub fn assert_eq<T>(v1: T, v2: T) where T: Eq {
    if (v1 != v2) {
        log(v1);
        log(v2);
        revert(FAILED_ASSERT_EQ_SIGNAL);
    }
}
