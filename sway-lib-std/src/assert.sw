library assert;

use ::revert::revert;
use ::context::call_frames;

/// Assert that a value is true
pub fn assert(a: bool) {
    if !a {
        revert(0);
    } else {
        ()
    }
}

/// wrapper for `assert` that allows passing a custom revert value `v` if condition `c` is not true.
pub fn require<T>(c: bool, v: T) {
    if !c {
        if is_reference_type() {
            let size = size_of();
            let this = contract_id();
            asm(r1: T, r2: size, r3: this) {
                logd r3 zero r1 r2;
            };
        }
        log(T)
        revert(v)
    } else {
        ()
    }
}
