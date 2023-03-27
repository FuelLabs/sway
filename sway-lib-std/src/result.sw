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
        let something = Result::Ok(42);
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
