//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::intrinsics::size_of_val;
use ::convert::From;
use ::hash::*;

/// The `EvmAddress` type, a struct wrapper around the inner `b256` value.
pub struct EvmAddress {
    /// The underlying evm address data.
    bits: b256,
}

impl EvmAddress {
    /// Returns the underlying bits for the EvmAddress type.
    ///
    /// # Returns
    ///
    /// * [b256] - The `b256` that make up the EvmAddress.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::evm::EvmAddress;
    ///
    /// fn foo() {
    ///     let evm_address = EvmAddress::zero();
    ///     assert(evm_address.bits() == b256::zero());
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }

    /// Returns the zero value for the `EvmAddress` type.
    ///
    /// # Returns
    ///
    /// * [EvmAddress] -> The zero value for the `EvmAddress` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::evm::EvmAddress;
    ///
    /// fn foo() {
    ///     let zero_evm_address = EvmAddress::zero();
    ///     assert(zero_evm_address == EvmAddress::from(b256::zero()));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bits: b256::zero(),
        }
    }

    /// Returns whether an `EvmAddress` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `EvmAddress` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::evm::EvmAddress;
    ///
    /// fn foo() {
    ///     let zero_evm_address = EvmAddress::zero();
    ///     assert(zero_evm_address.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}

impl ::ops::Eq for EvmAddress {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
    }
}

/// Functions for casting between the `b256` and `EvmAddress` types.
impl From<b256> for EvmAddress {
    /// Casts raw `b256` data to an `EvmAddress`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [EvmAddress] - The newly created `EvmAddress` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::evm::EvmAddress;
    ///
    /// fn foo() {
    ///    let evm_address = EvmAddress::from(b256::zero());
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        // An EVM address is only 20 bytes, so the first 12 are set to zero
        // Create a mutable local copy of `bits`
        let mut local_bits = bits;
        asm(r1: local_bits) {
            mcli r1 i12;
        };

        Self {
            bits: local_bits,
        }
    }
}

impl From<EvmAddress> for b256 {
    /// Casts an `EvmAddress` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `EvmAddress`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::evm::EvmAddress;
    ///
    /// fn foo() {
    ///     let evm_address = EvmAddress::zero();
    ///     let b256_data: b256 = evm_address.into();
    ///     assert(b256_data == b256::zero());
    /// }
    /// ```
    fn from(addr: EvmAddress) -> b256 {
        addr.bits
    }
}

impl Hash for EvmAddress {
    fn hash(self, ref mut state: Hasher) {
        let Address { bits } = self;
        bits.hash(state);
    }
}
