library revert;

use ::logging::log;
use ::error_signals::FAILED_REQUIRE_SIGNAL;

/// Context-dependent:
/// will panic if used in a predicate
/// will revert if used in a contract
///
/// ### Arguments
///
/// * `code` - The code with which to revert the program
///
/// ### Reverts
///
/// Reverts when called in a contract
///
/// # Panics
///
/// Panics when called in a predicate
///
/// ### Examples
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
/// ### Arguments
///
/// * `condition` - The condition upon which to decide whether to revert or not
/// * `value` - The value which will be logged in case `condition` is `false`
///
/// ### Reverts
///
/// Reverts when `condition` is false
///
/// ### Examples
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
