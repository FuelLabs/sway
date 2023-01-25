library contract_id;
//! A wrapper around the `b256` type to help enhance type-safety.
use ::assert::assert;
use ::bytes::Bytes;
use ::convert::From;
use ::option::Option;

/// The `ContractId` type, a struct wrappper around the inner `b256` value.
pub struct ContractId {
    value: b256,
}

impl core::ops::Eq for ContractId {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `ContractId` types.
impl From<b256> for ContractId {
    fn from(bits: b256) -> ContractId {
        ContractId { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}
