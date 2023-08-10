//! Functions to panic or revert with a given error code.
library;

use ::logging::log;
use ::error_signals::FAILED_REQUIRE_SIGNAL;

/// Will either panic or revert with a given number depending on the context.
///
/// # Additional Information
///
/// If used in a predicate, it will panic.
/// If used in a contract, it will revert.
///
/// # Arguments
///
/// * `code`: [u64] - The code with which to revert the program.
///
/// # Reverts
///
/// * Reverts unconditionally.
///
/// # Examples
///
/// ```sway
/// fn foo(should_revert: bool) {
///     match should_revert {
///         true => revert(0),
///         false => {},
///     }
/// }
/// ```
pub fn revert(code: u64) {
    __revert(code)
}

/// Checks if the given `condition` is `true` and if not, logs `value` and reverts.
///
/// # Arguments
///
/// * `condition`: [bool] - The condition upon which to decide whether to revert or not.
/// * `value`: [T] - The value which will be logged in case `condition` is `false`.
///
/// # Reverts
///
/// * Reverts when `condition` is `false`.
///
/// # Examples
///
/// ```sway
/// fn foo(a: u64, b: u64) {
///     require(a == b, "a was not equal to b");
///     // If the condition was true, code execution will continue
///     log("The require function did not revert");
/// }
/// ```
pub fn require<T>(condition: bool, value: T) {
    if !condition {
        log(value);
        revert(FAILED_REQUIRE_SIGNAL)
    }
}
