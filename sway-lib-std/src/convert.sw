//! Traits for conversions between types.
library;

use ::option::Option;

/// Used to do value-to-value conversions.
pub trait From<T> {
    /// Converts to this type from the input type.
    fn from(b: T) -> Self;
}

/// Used to do value-to-value conversions.
pub trait Into<T> {
    /// Converts this type into the (usually inferred) input type.
    fn into(self) -> T;
}

impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}

impl<T> From<T> for T {
    fn from(t: T) -> T {
        t
    }
}

// TODO: return a Result when https://github.com/FuelLabs/sway/issues/610 is resolved
/// Used to attempt to do value-to-value conversions.
/// Returns None if the conversion can't be performed in a lossless manner.
pub trait TryFrom<T> {
    /// Performs the conversion. Returns None if the conversion can't be performed in a lossless manner.
    fn try_from(b: T) -> Option<Self>;
}

pub trait TryInto<T> {
    fn try_into(self) -> Option<T>;
}

impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
{
    fn try_into(self) -> Option<U> {
        U::try_from(self)
    }
}

impl<T> TryFrom<T> for T {
    fn try_from(t: T) -> Option<T> {
        Option::Some(t)
    }
}
