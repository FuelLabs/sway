library eth_address;

//! A wrapper around the b256 type to help enhance type-safety.

/// The Address type, a struct wrappper around the inner `value`.
pub struct EthAddress {
    value: b256,
}

impl core::ops::Eq for EthAddress {
    fn eq(self, other: Self) -> bool {
        // An `Address` in Sway is 32 bytes
        asm(r1: self, r2: other, result, bytes_to_compare: 32) {
            meq result r1 r2 bytes_to_compare;
            result: bool
        }
    }
}

pub trait From {
    fn from(b: b256) -> Self;
} {
    fn into(addr: EthAddress) -> b256 {
        addr.value
    }
}

/// Functions for casting between the b256 and Address types.
impl From for EthAddress {
    fn from(bits: b256) -> EthAddress {

        // An ethereum address is only 20 bytes, so the first 12 are set to zero
        asm(r1: bits) {
            mcli r1 i12;
        };

        EthAddress {
            value: bits,
        }
    }
}
