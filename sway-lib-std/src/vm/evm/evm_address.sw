library evm_address;

//! A wrapper around the `b256` type to help enhance type-safety.
use ::intrinsics::size_of_val;
use ::convert::From;

/// The `EvmAddress` type, a struct wrappper around the inner `b256` value.
pub struct EvmAddress {
    value: b256,
}

impl core::ops::Eq for EvmAddress {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256l` and `EvmAddress` types.
impl From<b256> for EvmAddress {
    fn from(bits: b256) -> EvmAddress {
        // An EVM address is only 20 bytes, so the first 12 are set to zero
        // Create a mutable local copy of `bits`
        let mut local_bits = bits;
        asm(r1: local_bits) {
            mcli r1 i12;
        };

        EvmAddress {
            value: local_bits,
        }
    }

    fn into(self) -> b256 {
        self.value
    }
}
