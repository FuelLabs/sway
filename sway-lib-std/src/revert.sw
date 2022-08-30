library revert;

use ::logging::log;

const FAILED_REQUIRE_SIGNAL = 42;

/// Context-dependent:
/// will panic if used in a predicate
/// will revert if used in a contract
pub fn revert(code: u64) {
    asm(r1: code) {
        rvrt r1;
    }
}

/// Checks if the given `condition` is `true` and if not, logs `value` and reverts.
pub fn require<T>(condition: bool, value: T) {
    if !condition {
        log(value);
        revert(FAILED_REQUIRE_SIGNAL)
    }
}
