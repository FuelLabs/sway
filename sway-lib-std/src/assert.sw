library assert;

use ::revert::revert;


/// Assert that a value is true
pub fn assert(a: bool) {
    if !a {
        revert(0);
    } else {
        ()
    }
}

/// A wrapper for `assert` that allows logging a custom value `v` if condition `c` is not true.
/// This will then revert with the value `42`, which indicates that you should look at the previous logd receipt for further debugging clues.
pub fn require<T>(c: bool, v: T) {
    if !c {
        let size = size_of::<T>();
        asm(r1: v, r2: size) {
            logd zero zero r1 r2;
        };
        revert(42)
    } else {
        ()
    }
}
