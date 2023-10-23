//! Traits for conversions between types.
library;

use ::option::Option;

/// Used to do value-to-value conversions.
pub trait From<T> {
    /// Converts to this type from the input type.
    fn from(b: T) -> Self;
    /// Converts this type into the (usually inferred) input type.
    fn into(self) -> T;
}

// TODO: return a Result when https://github.com/FuelLabs/sway/issues/610 is resolved
/// Used to attempt to do value-to-value conversions.
/// Returns None if the conversion can't be performed in a lossless manner.
pub trait TryFrom<T> {
    /// Performs the conversion. Returns None if the conversion can't be performed in a lossless manner.
    fn try_from(b: T) -> Option<Self>;
}
