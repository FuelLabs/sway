//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::From;
use ::hash::{Hash, Hasher};
use ::math::Zero;

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
    /// fn foo() -> {
    ///     let my_address = Address::zero();
    ///     assert(my_address.bits() == b256::zero());
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
    /// fn foo() {
    ///    let address = Address::from(b256::zero());
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
    /// fn foo() {
    ///     let address = Address::zero();
    ///     let b256_data = address.into();
    ///     assert(b256_data == b256::zero());
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

impl Zero for Address {
    fn zero() -> Self {
        Self {
            bits: b256::zero()
        }
    }

    fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}
