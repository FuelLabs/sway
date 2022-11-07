library address;

use ::convert::From;

//! A wrapper around the b256 type to help enhance type-safety.
/// The `Address` type, a struct wrappper around the inner `b256` value.
pub struct Address {
    value: b256,
}

impl core::ops::Eq for Address {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `Address` types.
impl From<b256> for Address {
    fn from(bits: b256) -> Address {
        Address { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}
