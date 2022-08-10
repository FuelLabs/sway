//! Error handling with the `Option` type.
//!
//! [`Option<T>`][`Option`] is the type used for representing the existence or absence of a value. It is an enum with the variants, [`Some(T)`], representing
//! some value, and [`None()`], representing
//! no value.

library option;

use ::revert::revert;

/// `Option` is a type that represents either the existence of a value ([`Some`]) or a value's absence
/// ([`None`]).
pub enum Option<T> {
    /// Signifies the absence of a value
    None: (),

    /// Contains the value
    Some: T,
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
        match self {
            Option::Some(_) => {
                true
            },
            _ => {
                false
            },
        }
    }

    /// Returns `true` if the result is [`None`].
    fn is_none(self) -> bool {
        match self {
            Option::Some(_) => {
                false
            },
            _ => {
                true
            },
        }
    }

    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// Because this function may revert, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`None`]
    /// case explicitly.
    fn unwrap(self) -> T {
        match self {
            Option::Some(inner_value) => {
                inner_value
            },
            _ => {
                revert(0)
            },
        }
    }
}
