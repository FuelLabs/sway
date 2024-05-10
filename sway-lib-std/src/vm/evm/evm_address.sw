//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::intrinsics::size_of_val;
use ::convert::From;
use ::hash::*;
use ::math::Zero;

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
}

impl core::ops::Eq for EvmAddress {
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
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let evm_address = EvmAddress::from(ZERO_B256);
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
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let evm_address = EvmAddress::from(ZERO_B256);
    ///     let b256_data: b256 = evm_address.into();
    ///     assert(b256_data == ZERO_B256);
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

impl Zero for EvmAddress {
    fn zero() -> Self {
        Self {
            bits: b256::zero()
        }
    }

    fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}

#[test]
fn test_evm_address_from_b256() {
    use ::assert::assert;

    let evm_address = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(
        evm_address
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn test_evm_address_into_b256() {
    use ::assert::assert;
    use ::convert::Into;

    let evm_address = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data: b256 = evm_address.into();
    assert(b256_data == 0x0000000000000000000000000000000000000000000000000000000000000001);
}
