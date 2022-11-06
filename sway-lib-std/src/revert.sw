library revert;

use ::logging::log;
use ::error_signals::FAILED_REQUIRE_SIGNAL;

/// Context-dependent:
/// will panic if used in a predicate
/// will revert if used in a contract
pub fn revert(code: u64) {
    __revert(code)
}

/// Checks if the given `condition` is `true` and if not, logs `value` and reverts.
pub fn require<T>(condition: bool, value: T) {
    if !condition {
        log(value);
        revert(FAILED_REQUIRE_SIGNAL)
    }
}
