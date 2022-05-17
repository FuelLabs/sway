library assert;

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
        let ref_type = __is_reference_type::<T>();
        let size = __size_of::<T>();
        if ref_type {
            asm(r1: v, r2: size) {
                logd zero zero r1 r2;
            };
        } else {
            asm(r1: v) {
                log r1 zero zero zero;
            }
        }
        revert(FAILED_REQUIRE_SIGNAL)
    } else {
        ()
    }
}
