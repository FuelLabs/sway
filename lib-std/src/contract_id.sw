library contract_id;
//! A wrapper around the b256 type to help enhance type-safety.

/// The ContractId type, a struct wrappper around the inner `value`.
pub struct ContractId {
    value: b256,
}

// @todo make this generic when possible
pub trait From {
    fn from(b: b256) -> Self;
} {
    fn into(addr: ContractId) -> b256 {
        addr.value
    }
}

/// Functions for casting between the b256 and ContractId types.
impl From for ContractId {
    fn from(bits: b256) -> ContractId {
        ContractId {
            value: bits,
        }
    }
}
