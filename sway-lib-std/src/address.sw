//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::From;
use ::hash::{Hash, Hasher};

/// The `Address` type, a struct wrapper around the inner `b256` value.
pub struct Address {
    /// The underlying raw `b256` data of the address.
    bits: b256,
}

impl Address {
    /// Returns the underlying raw `b256` data of the address.
    ///
    /// # Returns
    ///
    /// * [b256] - The raw data of the address.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() -> {
    ///     let my_address = Address::from(ZERO_B256);
    ///     assert(my_address.bits() == ZERO_B256);
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }
}

impl core::ops::Eq for Address {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
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
        Self { bits }
    }
}

impl From<Address> for b256 {
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
    ///     let b256_data: b256 = address.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn from(address: Address) -> Self {
        address.bits()
    }
}

impl Hash for Address {
    fn hash(self, ref mut state: Hasher) {
        let Address { bits } = self;
        bits.hash(state);
    }
}

#[test]
fn test_address_from_b256() {
    use ::assert::assert;

    let my_address = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(my_address.bits() == 0x0000000000000000000000000000000000000000000000000000000000000001);
}

#[test]
fn test_address_into_b256() {
    use ::assert::assert;
    use ::convert::Into;

    let address = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data: b256 = address.into();
    assert(b256_data == 0x0000000000000000000000000000000000000000000000000000000000000001);
}
