//! A wrapper type with two variants, `Address` and `ContractId`.
//! The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library;

use ::assert::assert;
use ::address::Address;
use ::constants::{ZERO_B256, BASE_ASSET_ID};
use ::contract_id::ContractId;
use ::option::Option;

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

impl Identity {
    pub fn as_address(self) -> Option<Address> {
        match self {
            Identity::Address(address) => Option::Some(address),
            Identity::ContractId(_) => Option::None,
        }
    }

    pub fn as_contract_id(self) -> Option<ContractId> {
        match self {
            Identity::Address(_) => Option::None,
            Identity::ContractId(contract_id) => Option::Some(contract_id),
        }
    }

    pub fn is_address(self) -> bool {
        match self {
            Identity::Address(_) => true,
            Identity::ContractId(_) => false,
        }
    }

    pub fn is_contract_id(self) -> bool {
        match self {
            Identity::Address(_) => false,
            Identity::ContractId(_) => true,
        }
    }

    pub fn value(self) -> b256 {
        match self {
            Identity::Address(address) => address.value,
            Identity::ContractId(contract_id) => contract_id.value,
        }
    }
}

#[test]
fn test_address() {
    let address = Address::from(ZERO_B256);
    let identity = Identity::Address(address);
    assert(identity.is_address());
    assert(!identity.is_contract_id());
    assert(identity.as_address().unwrap() == address);
    assert(identity.as_contract_id().is_none());
    assert(identity.value() == ZERO_B256);
}

#[test]
fn test_contract_id() {
    let contract_id = BASE_ASSET_ID;
    let identity = Identity::ContractId(contract_id);
    assert(!identity.is_address());
    assert(identity.is_contract_id());
    assert(identity.as_contract_id().unwrap() == contract_id);
    assert(identity.as_address().is_none());
    assert(identity.value() == ZERO_B256);
}
