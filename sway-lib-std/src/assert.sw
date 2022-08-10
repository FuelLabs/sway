library assert;

use ::revert::revert;

/// Asserts that the given `condition` will always be `true` during runtime.
/// To check for conditions that may not be `true`, use `std::revert::require` instead.
/// See: https://en.wikipedia.org/wiki/Assertion_(software_development)#Comparison_with_error_handling
pub fn assert(condition: bool) {
    if !condition {
        revert(0);
    }
}
