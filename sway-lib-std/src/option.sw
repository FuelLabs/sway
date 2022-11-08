library option;

//! Optional values.
//!
//! Type [`Option`] represents an optional value: every [`Option`]
//! is either [`Some`] and contains a value, or [`None`], and
//! does not. [`Option`] types are very common in Rust code, as
//! they have a number of uses:
//!
//! * Initial values
//! * Return value for otherwise reporting simple errors, where [`None`] is
//!   returned on error
//! * Optional struct fields
//! * Optional function arguments
//!
//! [`Option`]s are commonly paired with pattern matching to query the presence
//! of a value and take action, always accounting for the [`None`] case.
//!
//! ```
//! fn divide(numerator: u64, denominator: u64) -> Option<u64> {
//!     if denominator == 0 {
//!         Option::None
//!     } else {
//!         Option::Some(numerator / denominator)
//!     }
//! }
//!
//! // The return value of the function is an option
//! let result = divide(6, 2);
//!
//! // Pattern match to retrieve the value
//! match result {
//!     // The division was valid
//!     Option::Some(x) => std::logging::log(x),
//!     // The division was invalid
//!     Option::None    => std::logging::log("Cannot divide by 0"),
//! }
//! ```
//!
//! # Method overview
//!
//! In addition to working with pattern matching, [`Option`] provides a wide
//! variety of different methods.
//!
//! ## Querying the variant
//!
//! The [`is_some`] and [`is_none`] methods return [`true`] if the [`Option`]
//! is [`Some`] or [`None`], respectively.
//!
//! [`is_none`]: Option::is_none
//! [`is_some`]: Option::is_some
//!
//! ## Extracting the contained value
//!
//! These methods extract the contained value in an [`Option<T>`] when it
//! is the [`Some`] variant. If the [`Option`] is [`None`]:
//!
//! * [`unwrap`] panics with a generic message
//! * [`unwrap_or`] returns the provided default value
//!
//! [`unwrap`]: Option::unwrap
//! [`unwrap_or`]: Option::unwrap_or
//!
//! ## Transforming contained values
//!
//! These methods transform [`Option`] to [`Result`]:
//!
//! * [`ok_or`] transforms [`Some(v)`] to [`Ok(v)`], and [`None`] to
//!   [`Err(err)`] using the provided default `err` value
//! * [`transpose`] transposes an [`Option`] of a [`Result`] into a
//!   [`Result`] of an [`Option`]
//!
//! [`Err(err)`]: Err
//! [`Ok(v)`]: Ok
//! [`Some(v)`]: Some
//! [`ok_or`]: Option::ok_or
//! [`transpose`]: Option::transpose

use ::convert::From;
use ::revert::revert;
use ::result::Result;

/// The `Option` type. See [the module level documentation](self) for more.
pub enum Option<T> {
    /// No value.
    None: (),
    /// Some value of type `T`. 
    Some: T,
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////

impl<T> Option<T> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the option is a [`Some`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Option<u32> = Option::Some(2);
    /// assert(x.is_some());
    ///
    /// let x: Option<u32> = Option::None;
    /// assert(!x.is_some());
    /// ```
    pub fn is_some(self) -> bool {
        match self {
            Option::Some(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the option is a [`None`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// let x: Option<u32> = Option::Some(2);
    /// assert(!x.is_none());
    ///
    /// let x: Option<u32> = Option::None;
    /// assert(x.is_none());
    /// ```
    pub fn is_none(self) -> bool {
        match self {
            Option::Some(_) => false,
            _ => true,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Getting to contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns the contained [`Some`] value, consuming the `self` value.
    ///
    /// Because this function may revert, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`None`]
    /// case explicitly, or call [`unwrap_or`].
    ///
    /// [`unwrap_or`]: Option::unwrap_or
    ///
    /// # Reverts 
    ///
    /// Reverts if the self value equals [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Option::Some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```should_panic
    /// let x: Option<&str> = Option::None;
    /// assert_eq!(x.unwrap(), "air"); // fails
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Option::Some(inner_value) => inner_value,
            _ => revert(0),
        }
    }


    /// Returns the contained [`Some`] value or a provided default.
    ///
    /// [`unwrap_or`]: Option::unwrap_or
    ///
    /// # Examples
    ///
    /// ```
    /// assert(Option::Some(42).unwrap_or(69), 42);
    /// assert(Option::None.unwrap_or(69), 69);
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Option::Some(x) => x,
            Option::None => default,
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////
    
    /// Transforms the `Option<T>` into a [`Result<T, E>`], mapping [`Some(v)`] to
    /// [`Ok(v)`] and [`None`] to [`Err(err)`].
    ///
    /// [`Ok(v)`]: Ok
    /// [`Err(err)`]: Err
    /// [`Some(v)`]: Some
    /// [`ok_or`]: Option::ok_or
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Option::Some(42);
    /// match x.ok_or(0) {
    ///     Result::Ok(inner) => assert(inner == 42),
    ///     Result::Err => revert(0),
    /// }
    ///
    /// let x:Option<u64> = Option::None;
    /// match x.ok_or(0) {
    ///     Result::Ok(_) => revert(0),
    ///     Result::Err(e) => assert(e == 0),
    /// }
    /// ```
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Option::Some(v) => Result::Ok(v),
            Option::None => Result::Err(err),
        }
    }
}
