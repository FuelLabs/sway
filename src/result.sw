//! Error handling with the `Result` type.
//!
//! [`Result<T, E>`][`Result`] is the type used for returning and propagating
//! errors. It is an enum with the variants, [`Ok(T)`], representing
//! success and containing a value, and [`Err(E)`], representing error
//! and containing an error value.

library result;

/// `Result` is a type that represents either success ([`Ok`]) or failure
/// ([`Err`]).
pub enum Result<T, E> {
    /// Contains the success value
    Ok: T,

    /// Contains the error value
    Err: E,
}

/////////////////////////////////////////////////////////////////////////////
// Type implementation
/////////////////////////////////////////////////////////////////////////////

impl Result<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`].
    fn is_ok(self) -> bool {
        if let Result::Ok(t) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if the result is [`Err`].
    fn is_err(self) -> bool {
        if let Result::Ok(t) = self {
            false
        } else {
            true
        }
    }
}
