library evm_address;

//! A wrapper around the b256 type to help enhance type-safety.

use ::intrinsics::{addr_of, raw_eq, size_of_val};

/// The Address type, a struct wrappper around the inner `value`.
pub struct EvmAddress {
    value: b256,
}

impl core::ops::Eq for EvmAddress {
    fn eq(self, other: Self) -> bool {
        raw_eq(addr_of(self), addr_of(other), size_of_val(self))
    }
}

pub trait From {
    fn from(b: b256) -> Self;
} {
    fn into(addr: EvmAddress) -> b256 {
        addr.value
    }
}

/// Functions for casting between the b256 and Address types.
impl From for EvmAddress {
    fn from(bits: b256) -> EvmAddress {
        // An EVM address is only 20 bytes, so the first 12 are set to zero
        asm(r1: bits) {
            mcli r1 i12;
        };

        EvmAddress {
            value: bits,
        }
    }
}
