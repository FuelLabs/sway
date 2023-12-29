//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::From;
use ::hash::{Hash, Hasher};

/// The `Address` type, a struct wrapper around the inner `b256` value.
pub struct Address {
    /// The underlying raw `b256` data of the address.
    pub value: b256,
}

impl core::ops::Eq for Address {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `Address` types.
impl From<b256> for Address {
    /// Casts raw `b256` data to an `Address`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [Address] - The newly created `Address` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let address = Address::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { value: bits }
    }

    /// Casts an `Address` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `Address`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let address = Address::from(ZERO_B256);
    ///     let b256_data = address.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn into(self) -> b256 {
        self.value
    }
}

impl Hash for Address {
    fn hash(self, ref mut state: Hasher) {
        let Address { value } = self;
        value.hash(state);
    }
}
