//! Traits for conversions between types.
library convert;

use ::option::Option;

/// Used to do value-to-value conversions while consuming the input value.
pub trait From<T> {
    fn from(b: T) -> Self;
    fn into(self) -> T;
}

pub trait TryFrom<T> {
    fn try_from(b: T) -> Option<Self>;
}
