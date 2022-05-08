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

/// wrapper for `assert` that allows passing a custom revert value `v` if condition `c` is not true.
pub fn require(c: bool, v: u64) {
    if !c {
        revert(v)
    } else {
        ()
    }
}
