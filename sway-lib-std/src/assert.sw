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
