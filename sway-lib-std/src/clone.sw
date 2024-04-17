//! The clone trait, for explicit duplication.
library;

/// A common trait for the ability to explicitly duplicate an object.
pub trait Clone {
    /// Clone self into a new value of the same type.
    fn clone(self) -> Self;
}
