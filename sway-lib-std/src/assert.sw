library assert;

use ::panic::panic;

/// Assert that a value is true
pub fn assert(a: bool) {
    if !a {
        panic(0);
    } else {
        ()
    }
}
