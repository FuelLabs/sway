library;

use ::option::Option::{self, *};

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
        let source = self.as_ptr();

        if self.len() == str_size {
            let s: S = asm(str_size: str_size, source: source, dest) {
                move dest sp;
                cfe str_size;
                mcp dest source str_size;
                dest: S
            };
            asm(str_size: str_size) {
                cfs str_size;
            }
            Some(s)
        } else {
            None
        }
    }
}

#[test]
fn str_slice_to_str_array() {
    use ::assert::*;
    use core::str::*;

    let a = "abcd";
    let b: str[4] = a.try_as_str_array().unwrap();
    let c = from_str_array(b);

    assert(a == c);
}
