library address;
//! A wrapper around the b256 type to help enhance type-safety.

/// The Address type, a struct wrappper around the inner `value`.
pub struct Address {
    value: b256,
}

impl core::ops::Eq for Address {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

pub trait From {
    fn from(b: b256) -> Self;
    fn into(self) -> b256;
}

/// Functions for casting between the b256 and Address types.
impl From for Address {
    fn from(bits: b256) -> Address {
        Address { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}
