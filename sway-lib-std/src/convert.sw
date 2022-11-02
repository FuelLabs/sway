//! Traits for conversions between types.
library convert;

/// Used to do value-to-value conversions while consuming the input value.
pub trait From {
    fn from(b: T) -> Self;
    fn into(self) -> T;
}
