library assert;

use ::logging::log;
use ::revert::revert;

const FAILED_REQUIRE_SIGNAL = 42;

/// Assert that a value is true
pub fn assert(a: bool) {
    if !a {
        revert(0);
    } else {
        ()
    }
}

/// A wrapper for `assert` that allows logging a custom value `v` if condition `c` is not true.
pub fn require<T>(c: bool, v: T) {
    if !c {
        log(v);
        revert(FAILED_REQUIRE_SIGNAL)
    } else {
        ()
    }
}
