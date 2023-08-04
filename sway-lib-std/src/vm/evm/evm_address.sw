//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::intrinsics::size_of_val;
use ::convert::From;

/// The `EvmAddress` type, a struct wrapper around the inner `b256` value.
pub struct EvmAddress {
    /// The underlying evm address data.
    value: b256,
}

impl core::ops::Eq for EvmAddress {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `EvmAddress` types.
impl From<b256> for EvmAddress {
    fn from(bits: b256) -> Self {
        // An EVM address is only 20 bytes, so the first 12 are set to zero
        // Create a mutable local copy of `bits`
        let mut local_bits = bits;
        asm(r1: local_bits) {
            mcli r1 i12;
        };

        Self {
            value: local_bits,
        }
    }

    fn into(self) -> b256 {
        self.value
    }
}
