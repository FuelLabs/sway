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
library result;

use ::revert::revert;

/// `Result` is a type that represents either success (`Ok`) or failure (`Err`).
pub enum Result<T, E> {
    /// Contains the success value.
    Ok: T,
    /// Contains the error value.
    Err: E,
}

// Type implementation
//
impl<T, E> Result<T, E> {
    // Querying the contained values
    //
    /// Returns `true` if the result is `Ok`.
    ///
    /// ### Examples
    ///
    /// ```
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// let x: Result<u64, Error> = Result::Ok(42);
    /// assert(x.is_ok());
    ///
    /// let y: Result<u64, Error> = Result::Err(Error::NotFound));
    /// assert(!x.is_ok());
    /// ```
    pub fn is_ok(self) -> bool {
        match self {
            Result::Ok(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the result is `Err`.
    ///
    /// ### Examples
    ///
    /// ```
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// let x: Result<u64, Error> = Result::Ok(42);
    /// assert(!x.is_err());
    ///
    /// let y: Result<u64, Error> = Result::Err(Error::NotFound));
    /// assert(x.is_err());
    /// ```
    pub fn is_err(self) -> bool {
        match self {
            Result::Ok(_) => false,
            _ => true,
        }
    }

    /// Returns the contained `Ok` value, consuming the `self` value.
    ///
    /// Because this function may revert, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the `Err`
    /// case explicitly.
    ///
    /// ### Reverts
    ///
    /// Reverts if the self value is `Err`.
    ///
    /// ### Examples
    ///
    /// ```
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// let x: Result<u64, Error> = Result::Ok(42);
    /// assert(x.unwrap() == 42);
    ///
    /// let y: Result<u64, Error> = Result::Err(Error::NotFound));
    /// assert(x.unwrap() == 42); // reverts
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Result::Ok(inner_value) => inner_value,
            _ => revert(0),
        }
    }

    /// Returns the contained `Ok` value or a provided default.
    ///
    /// ### Examples
    ///
    /// ```
    /// enum Error {
    ///     NotFound,
    ///     Invalid,
    /// }
    ///
    /// let x: Result<u64, Error> = Result::Ok(42);
    /// assert(x.unwrap_or(69) == 42);
    ///
    /// let y: Result<u64, Error> = Result::Err(Error::NotFound));
    /// assert(x.unwrap_or(69) == 69);
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result::Ok(inner_value) => inner_value,
            Result::Err(_) => default,
        }
    }

    // TODO: Implement the following transforms when Option and Result can
    // import one another:
    // - `ok(self) -> Option<T>`
    // - `err(self) -> Option<E>`
}
