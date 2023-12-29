library;

use ::ops::{Eq, Not, Ord};

/// `Never` represents the type of computations which never resolve to any value at all.
///
/// # Additional Information
///
/// `break`, `continue` and `return` expressions also have type `Never`. For example we are allowed to
/// write:
///
/// ```sway
/// let x: Never = {
///     return 123
/// };
/// ```
///
/// Although the `let` is pointless here, it illustrates the meaning of `Never`. Since `x` is never
/// assigned a value (because `return` returns from the entire function), `x` can be given type
/// `Never`. We could also replace `return 123` with a `revert()` or a never-ending `loop` and this code
/// would still be valid.
///
/// A more realistic usage of `Never` is in this code:
///
/// ```sway
/// let num: u32 = match get_a_number() {
///     Some(num) => num,
///     None => break,
/// };
/// ```
///
/// Both match arms must produce values of type [`u32`], but since `break` never produces a value
/// at all we know it can never produce a value which isn't a [`u32`]. This illustrates another
/// behaviour of the `Never` type - expressions with type `Never` will coerce into any other type.
///
/// Note that `Never` type coerces into any other type, another example of this would be:
///
/// ```sway
/// let x: u32 = {
///     return 123
/// };
/// ```
///
/// Regardless of the type of `x`, the return block of type `Never` will always coerce into `x` type.
///
/// # Examples
///
/// ```sway
/// fn foo() {
///     let num: u64 = match Option::None::<u64> {
///         Some(num) => num,
///         None => return,
///     };
/// }
/// ```
pub enum Never {}

impl Not for Never {
    fn not(self) -> Self {
        match self {}
    }
}

impl Eq for Never {
    fn eq(self, _other: Self) -> bool {
        self
    }
}

impl Ord for Never {
    fn gt(self, _other: Self) -> bool {
        self
    }
    fn lt(self, _other: Self) -> bool {
        self
    }
}
