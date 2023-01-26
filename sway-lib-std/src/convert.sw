//! Traits for conversions between types.
library convert;

use ::option::Option;

/// Used to do value-to-value conversions.
pub trait From<T> {
    fn from(b: T) -> Self;
    fn into(self) -> T;
}

// TODO: return a Result when https://github.com/FuelLabs/sway/issues/610 is resolved
/// Used to attempt to do value-to-value conversions.
/// Returns Option::None if the conversion can't be performed in a lossless manner.
pub trait TryFrom<T> {
    fn try_from(b: T) -> Option<Self>;
}
