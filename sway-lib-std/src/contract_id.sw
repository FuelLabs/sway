library contract_id;
//! A wrapper around the b256 type to help enhance type-safety.
use ::intrinsics::size_of_val;
use ::mem::{addr_of, eq};

/// The ContractId type, a struct wrappper around the inner `value`.
pub struct ContractId {
    value: b256,
}

impl core::ops::Eq for ContractId {
    fn eq(self, other: Self) -> bool {
        eq(addr_of(self), addr_of(other), size_of_val(self))
    }
}

// TODO: make this a generic trait. tracked here: https://github.com/FuelLabs/sway-lib-std/issues/58
pub trait From {
    fn from(b: b256) -> Self;
    fn into(self) -> b256;
}

/// Functions for casting between the b256 and ContractId types.
impl From for ContractId {
    fn from(bits: b256) -> ContractId {
        ContractId { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}
