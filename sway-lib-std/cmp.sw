//! A utility library for comparing values.
library;

/// A common trait for comparing values.
pub trait Cmp {
    /// Compares and returns the minimum of two values.
    fn min(self, other: Self) -> Self;
    /// Compares and returns the maximum of two values.
    fn max(self, other: Self) -> Self;
}