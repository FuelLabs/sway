//! A type for optional values.
//!
//! Type `Option` represents an optional value: every `Option`
//! is either `Some` and contains a value, or `None`, and
//! does not. `Option` types are very common in Sway code, as
//! they have a number of uses:
//!
//! * Initial values where `None` can be used as an initializer.
//! * Return value for otherwise reporting simple errors, where `None` is
//!   returned on error.
//! * Optional struct fields.
//! * Optional function arguments.
//!
//! `Option`s are commonly paired with pattern matching to query the presence
//! of a value and take action, always accounting for the `None` case.
//!
//! ```
//! fn divide(numerator: u64, denominator: u64) -> Option<u64> {
//!     if denominator == 0 {
//!         None
//!     } else {
//!         Some(numerator / denominator)
//!     }
//! }
//!
//! fn call_divide() {
//!     // The return value of the function is an option
//!     let result = divide(6, 2);
//!
//!     // Pattern match to retrieve the value
//!     match result {
//!         // The division was valid
//!         Some(x) => std::logging::log(x),
//!         // The division was invalid
//!         None    => std::logging::log("Cannot divide by 0"),
//!     }
//! }
//! ```
//!
//! # Method overview
//!
//! In addition to working with pattern matching, `Option` provides a wide
//! variety of different methods.
//!
//! # Querying the variant
//!
//! The `is_some` and `is_none` methods return `true` if the `Option`
//! is `Some` or `None`, respectively.
//!
//! `is_none`: `Option::is_none`
//! `is_some`: `Option::is_some`
//!
//! # Extracting the contained value
//!
//! These methods extract the contained value in an `Option<T>` when it
//! is the `Some` variant. If the `Option` is `None`:
//!
//! * `unwrap` reverts.
//! * `unwrap_or` returns the provided default value.
//!
//! `unwrap`   : `Option::unwrap`
//! `unwrap_or`: `Option::unwrap_or`
//!
//! # Transforming contained values
//!
//! These methods transform `Option` to `Result`:
//!
//! * `ok_or` transforms `Some(v)` to `Ok(v)`, and `None` to
//!   `Err(e)` using the provided default error value.
//!
//! `Err(e)` : `Result::Err`
//! `Ok(v)`  : `Result::Ok`
//! `Some(v)`: `Option::Some`
//! `ok_or`  : `Option::ok_or`
library;

use ::logging::log;
use ::result::Result;
use ::revert::revert;
use ::codec::*;
use ::debug::*;
use ::ops::*;

// ANCHOR: docs_option
/// A type that represents an optional value, either `Some(val)` or `None`.
pub enum Option<T> {
    /// No value.
    None: (),
    /// Some value of type `T`.
    Some: T,
}
// ANCHOR_END: docs_option

impl<T> PartialEq for Option<T>
where
    T: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Option::Some(a), Option::Some(b)) => a == b,
            (Option::None, Option::None) => true,
            _ => false,
        }
    }
}
impl<T> Eq for Option<T>
where
    T: Eq,
{}

// Type implementation
//
impl<T> Option<T> {
    // Querying the contained values
    //
    /// Returns whether the option is the `Some` variant.
    ///
    /// # Returns
    ///
    /// * [bool] - Returns `true` if the option is `Some`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: Option<u32> = Some(2);
    ///     assert(x.is_some());
    ///
    ///     let x: Option<u32> = None;
    ///     assert(!x.is_some());
    /// }
    /// ```
    pub fn is_some(self) -> bool {
        match self {
            Self::Some(_) => true,
            _ => false,
        }
    }

    /// Returns whether the option is the `None` variant.
    ///
    /// # Returns
    ///
    /// * [bool] - Returns `true` if the option is `None`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: Option<u32> = Some(2);
    ///     assert(!x.is_none());
    ///
    ///     let x: Option<u32> = None;
    ///     assert(x.is_none());
    /// }
    /// ```
    pub fn is_none(self) -> bool {
        match self {
            Self::Some(_) => false,
            _ => true,
        }
    }

    // Getting to contained values
    //
    /// Returns the contained `Some` value, consuming the `self` value.
    ///
    /// # Additional Information
    ///
    /// Because this function may revert, its use is generally discouraged.
    /// Instead, use pattern matching and handle the `None`
    /// case explicitly, or call `unwrap_or`.
    ///
    /// # Returns
    ///
    /// * [T] - The value contained by the option.
    ///
    /// # Reverts
    ///
    /// * Reverts if the `Option` is the `None` variant.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x = Some(42);
    ///     assert(x.unwrap() == 42);
    /// }
    /// ```
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: Option<u64> = None;
    ///     let value = x.unwrap(); // reverts
    /// }
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Self::Some(inner_value) => inner_value,
            _ => revert(0),
        }
    }

    /// Returns the contained `Some` value or a provided default.
    ///
    /// # Arguments
    ///
    /// * `default`: [T] - The default value the function will revert to.
    ///
    /// # Returns
    ///
    /// * [T] - The contained value or the default value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     assert(Some(42).unwrap_or(69) == 42);
    ///     assert(None::<u64>().unwrap_or(69) == 69);
    /// }
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Some(x) => x,
            Self::None => default,
        }
    }

    // Transforming contained values
    //
    /// Transforms the `Option<T>` into a `Result<T, E>`, mapping `Some(v)` to
    /// `Ok(v)` and `None` to `Err(e)`.
    ///
    /// # Additional Information
    ///
    /// `Ok(v)`  : `Result::Ok`
    /// `Err(e)` : `Result::Err`
    /// `Some(v)`: `Option::Some`
    /// `ok_or`  : `Option::ok_or`
    ///
    /// # Arguments
    ///
    /// * `err`: [E] - The error value if the option is `None`.
    ///
    /// # Returns
    ///
    /// * [Result<T, E>] - The result containing the value or the error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x = Some(42);
    ///     match x.ok_or(0) {
    ///         Result::Ok(inner) => assert(inner == 42),
    ///         Result::Err => revert(0),
    ///     }
    ///
    ///     let x: Option<u64> = None;
    ///     match x.ok_or(0) {
    ///         Result::Ok(_) => revert(0),
    ///         Result::Err(e) => assert(e == 0),
    ///     }
    /// }
    /// ```
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Self::Some(v) => Result::Ok(v),
            Self::None => Result::Err(err),
        }
    }

    /// Returns the contained `Some` value, consuming the `self` value.
    /// If the `Option` is the `None` variant, logs the provided message.
    ///
    /// # Additional Information
    ///
    /// Because this function may revert, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the `None`
    /// case explicitly.
    ///
    /// # Arguments
    ///
    /// * `msg`: [M] - The message to be logged if the `Option` is the `None` variant.
    ///
    /// # Returns
    ///
    /// * [T] - The value contained by the option.
    ///
    /// # Reverts
    ///
    /// * Reverts if the `Option` is the `None` variant.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let x: Option<u64> = Some(42);
    ///     assert(x.expect("X is known to be 42") == 42);
    ///
    ///     let y: Option<u64> = None;
    ///     let val = y.expect("Testing expect"); // reverts with `("Testing Expect")`
    /// }
    /// ```
    ///
    /// # Recommended Message Style
    ///
    /// We recommend that `expect` messages are used to describe the reason you *expect* the `Option` should be `Some`.
    ///
    /// ```sway
    /// let x: Option<u64> = bar(1);
    /// let value = x.expect("bar() should never return None with 1 as an argument");
    /// ```
    pub fn expect<M>(self, msg: M) -> T
    where
        M: AbiEncode,
    {
        match self {
            Self::Some(v) => v,
            Self::None => {
                log(msg);
                revert(0);
            },
        }
    }
}
