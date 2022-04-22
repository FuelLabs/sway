//! Error handling with the `Option` type.
//!
//! [`Option<T>`][`Option`] is the type used for representing the existence or absence of a value. It is an enum with the variants, [`Some(T)`], representing
//! some value, and [`None()`], representing
//! no value.

library option;

use ::panic::panic;

/// `Option` is a type that represents either the existence of a value ([`Some`]) or a value's absence
/// ([`None`]).
pub enum Option<T> {
    /// Contains the value
    Some: T,

    /// Signifies the absence of a value
    None: (),
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////

impl<T> Option<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Some`].
    fn is_some(self) -> bool {
        if let Option::Some(t) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if the result is [`None`].
    fn is_none(self) -> bool {
        if let Option::Some(t) = self {
            false
        } else {
            true
        }
    }

    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`None`]
    /// case explicitly.
    fn unwrap(self) -> T {
        if let Option::Some(inner_value) = self {
            inner_value
        } else {
            panic(0);
        }
    }
}
