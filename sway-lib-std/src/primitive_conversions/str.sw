library;

use ::option::Option::{self, *};
use ::str::*;
use ::ops::*;

impl str {
    /// Attempts to convert the string slice into a string array.
    ///
    /// # Returns
    ///
    /// * [Option<S>] - `Some(str_array)` if the lengths of the `S` str_array type and the string slice's lengths match. Else `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let string_slice = "abcd";
    ///     let string_array: str[4] = a.try_as_str_array().unwrap();
    /// }
    /// ```
    pub fn try_as_str_array<S>(self) -> Option<S> {
        __assert_is_str_array::<S>();
        let str_size = __size_of_str_array::<S>();
        let tmp_alloc_size = __size_of::<S>();
        let source = self.as_ptr();

        if self.len() == str_size {
            let s: S = asm(
                str_size: str_size,
                tmp_alloc_size: tmp_alloc_size,
                source: source,
                dest,
            ) {
                move dest sp;
                cfe tmp_alloc_size;
                mcp dest source str_size;
                dest: S
            };
            asm(tmp_alloc_size: tmp_alloc_size) {
                cfs tmp_alloc_size;
            }
            Some(s)
        } else {
            None
        }
    }
}
