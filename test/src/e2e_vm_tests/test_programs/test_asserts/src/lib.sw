//! A simple assert library to be used in test projects that need assert
//! functionality, but otherwise do not need to depend on the `std` library.
//!
//! To assert for equality, use the following patterns:
//!
//! ```ignore
//! assert_true(11, __eq(a, b)); // For asserting that `a` equals `b`.
//! assert_false(22, __eq(a, b)); // For asserting that `a` does not equal `b`.
//! ```
library;

pub fn assert_true(revert_code: u64, condition: bool) {
    if condition {
    } else {
        __revert(revert_code);
    }
}

pub fn assert_false(revert_code: u64, condition: bool) {
    if condition {
        __revert(revert_code);
    }
}
