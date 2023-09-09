//! Error handling with the `Result` type.
//!
//! `Result<T, E>` `Result` is the type used for returning and propagating
//! errors. It is an enum with the variants, `Ok(T)`, representing
//! success and containing a value, and `Err(E)`, representing error
//! and containing an error value.
//!
//! Functions return `Result` whenever errors are expected and recoverable. In
//! the `std` crate, `Result` is most prominently used for `Identity`
//! interactions and cryptographic operations.
//!
//! A simple function returning `Result` might be defined and used like so:
//!
//! ```
//! enum Version {
//!     Version1,
//!     Version2,
//! }
//!
//! enum VersionError {
//!     InvalidNumber,
//! }
//!
//! fn parse_version(version_number: u8) -> Result<Version, VersionError> {
//!     match version_number {
//!         1 => Ok(Version::Version1),
//!         2 => Ok(Version::Version2),
//!         _ => Err(VersionError::InvalidNumber),
//!     }
//! }
//! ```
//!
//! ### Method overview
//!
//! In addition to working with pattern matching, `Result` provides a variety
//! of methods.
//!
//! ### Querying the variant
//!
//! The `is_ok` and `is_err` methods return `true` if the `Result` is
//! `Ok` or `Err`, respectively.
//!
//! `is_ok` : `Result::is_ok`
//! `is_err`: `Result::is_err`
//!
//! ### Extracting the contained value
//!
//! These methods exctract the contained value in a `Result<T,E>` when it is
//! the `Ok` variant. If the `Result` is `Err`:
//!
//! * `unwrap` reverts.
//! * `unwrap_or` returns the default provided value.
//!
//! `unwrap`   : `Result::unwrap`
//! `unwrap_or`: `Result::unwrap_or`
library;

use ::revert::revert;

// ANCHOR: docs_result
/// `Result` is a type that represents either success (`Ok`) or failure (`Err`).
pub enum Result<T, E> {
    /// Contains the success value.
    Ok: T,
    /// Contains the error value.
    Err: E,
}
// ANCHOR_END: docs_result


// Type implementation
//
impl<T, E> Result<T, E> {
    // Querying the contained values
    //
    /// Returns whether a result contains a success value.
    ///
    /// # Returns
    ///
    /// * [bool] - Returns `true` if the result is `Ok`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// fn foo() {
    ///     let x: Result<u64, Error> = Result::Ok(42);
    ///     assert(x.is_ok());
    ///
    ///     let y: Result<u64, Error> = Result::Err(Error::NotFound));
    ///     assert(!y.is_ok());
    /// }
    /// ```
    pub fn is_ok(self) -> bool {
        match self {
            Self::Ok(_) => true,
            _ => false,
        }
    }

    /// Returns whether a result contains an error value.
    ///
    /// # Returns
    ///
    /// * [bool] - Returns `true` if the result is `Err`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// fn foo() {
    ///     let x: Result<u64, Error> = Result::Ok(42);
    ///     assert(!x.is_err());
    ///
    ///     let y: Result<u64, Error> = Result::Err(Error::NotFound));
    ///     assert(y.is_err());
    /// }
    /// ```
    pub fn is_err(self) -> bool {
        match self {
            Self::Ok(_) => false,
            _ => true,
        }
    }

    /// Returns the contained `Ok` value, consuming the `self` value.
    ///
    /// # Additional Information
    ///
    /// Because this function may revert, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the `Err`
    /// case explicitly.
    ///
    /// # Returns
    ///
    /// * [T] - The value contained by the result.
    ///
    /// # Reverts
    ///
    /// * Reverts if the `Result` is the `Err` variant.
    ///
    /// # Examples
    ///
    /// ```sway
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// fn foo() {
    ///     let x: Result<u64, Error> = Result::Ok(42);
    ///     assert(x.unwrap() == 42);
    ///
    ///     let y: Result<u64, Error> = Result::Err(Error::NotFound));
    ///     let val = y.unwrap(); // reverts
    /// }
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(inner_value) => inner_value,
            _ => revert(0),
        }
    }

    /// Returns the contained `Ok` value or a provided default.
    ///
    /// # Arguments
    ///
    /// * `default`: [T] - The value that is the default.
    ///
    /// # Returns
    ///
    /// * [T] - The value of the result or the default.
    ///
    /// # Examples
    ///
    /// ```sway
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// fn foo() {
    ///     let x: Result<u64, Error> = Result::Ok(42);
    ///     assert(x.unwrap_or(69) == 42);
    ///
    ///     let y: Result<u64, Error> = Result::Err(Error::NotFound));
    ///     assert(y.unwrap_or(69) == 69);
    /// }
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Ok(inner_value) => inner_value,
            Self::Err(_) => default,
        }
    }

    // TODO: Implement the following transforms when Option and Result can
    // import one another:
    // - `ok(self) -> Option<T>`
    // - `err(self) -> Option<E>`
}
