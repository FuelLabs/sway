library assert;

use ::revert::revert;

/// Asserts that the given `condition` will always be `true` during runtime.
/// To check for conditions that may not be `true`, use `std::revert::require` instead.
/// See: https://en.wikipedia.org/wiki/Assertion_(software_development)#Comparison_with_error_handling
///
/// ### Arguments
///
/// * `condition` - The condition which will be asserted to be true
///
/// ### Reverts
///
/// Reverts when `condition` is false
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
