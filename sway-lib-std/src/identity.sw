//! A wrapper type with two variants, `Address` and `ContractId`.
//! The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library;

use ::address::Address;
use ::contract_id::ContractId;

/// The `Identity` type: either an `Address` or a `ContractId`.
// ANCHOR: docs_identity
pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}
// ANCHOR_END: docs_identity
impl core::ops::Eq for Identity {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Identity::Address(address1), Identity::Address(address2)) => address1 == address2,
            (Identity::ContractId(asset1), Identity::ContractId(asset2)) => asset1 == asset2,
            _ => false,
        }
    }
}
