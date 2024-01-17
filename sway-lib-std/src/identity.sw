//! A wrapper type with two variants, `Address` and `ContractId`.
//! The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library;

use ::assert::assert;
use ::address::Address;
use ::alias::SubId;
use ::asset_id::AssetId;
use ::call_frames::contract_id;
use ::constants::{BASE_ASSET_ID, ZERO_B256};
use ::contract_id::ContractId;
use ::hash::{Hash, Hasher};
use ::option::Option::{self, *};
use ::predicate_id::PredicateId;

/// The `Identity` type: either an `Address` or a `ContractId`.
// ANCHOR: docs_identity
pub enum Identity {
    Address: Address,
    ContractId: ContractId,
    PredicateId: PredicateId,
}
// ANCHOR_END: docs_identity

impl core::ops::Eq for Identity {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Identity::Address(addr1), Identity::Address(addr2)) => addr1 == addr2,
            (Identity::ContractId(id1), Identity::ContractId(id2)) => id1 == id2,
            (Identity::PredicateId(pred1), Identity::PredicateId(pred2)) => pred1 == pred2,
            _ => false,
        }
    }
}

impl Identity {
    /// Returns the `Address` of the `Identity`.
    ///
    /// # Returns
    ///
    /// * [Option<Address>] - `Some(Address)` if the underlying type is an `Address`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let identity = Identity::Address(Address::from(ZERO_B256));
    ///     let address = identity.as_address();
    ///     assert(address == Address::from(ZERO_B256));
    /// }
    /// ```
    pub fn as_address(self) -> Option<Address> {
        match self {
            Self::Address(addr) => Some(addr),
            _ => None,
        }
    }

    /// Returns the `ContractId` of the `Identity`.
    ///
    /// # Returns
    ///
    /// * [Option<ContractId>] - `Some(Contract)` if the underlying type is an `ContractId`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let identity = Identity::ContractId(ContractId::from(ZERO_B256));
    ///     let contract_id = identity.as_contract_id();
    ///     assert(contract_id == ContractId::from(ZERO_B256));
    /// }
    /// ```
    pub fn as_contract_id(self) -> Option<ContractId> {
        match self {
            Self::ContractId(id) => Some(id),
            _ => None,
        }
    }

    /// Returns the `PredicateId` of the `Identity`.
    ///
    /// # Returns
    ///
    /// * [Option<PredicateId>] - `Some(PredicateId)` if the underlying type is an `PredicateId`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let identity = Identity::PredicateId(PredicateId::from(ZERO_B256));
    ///     let predicate_id = identity.as_address();
    ///     assert(predicate_id == PredicateId::from(ZERO_B256));
    /// }
    /// ```
    pub fn as_predicate_id(self) -> Option<PredicateId> {
        match self {
            Self::PredicateId(pred) => Some(pred),
            _ => None,
        }
    }

    /// Returns whether the `Identity` represents an `Address`.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the `Identity` holds an `Address`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let identity = Identity::Address(Address::from(ZERO_B256));
    ///     assert(identity.is_address());
    /// }
    /// ```
    pub fn is_address(self) -> bool {
        match self {
            Self::Address(_) => true,
            _ => false,
        }
    }

    /// Returns whether the `Identity` represents a `ContractId`.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the `Identity` holds a `ContractId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let identity = Identity::ContractId(ContractId::from(ZERO_B256));
    ///     assert(identity.is_contract_id());
    /// }
    /// ```
    pub fn is_contract_id(self) -> bool {
        match self {
            Self::ContractId(_) => true,
            _ => false,
        }
    }

    /// Returns whether the `Identity` represents an `PredicateId`.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the `Identity` holds an `v`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let identity = Identity::PredicateId(PredicateId::from(ZERO_B256));
    ///     assert(identity.is_predicate());
    /// }
    /// ```
    pub fn is_predicate_id(self) -> bool {
        match self {
            Self::PredicateId(_) => true,
            _ => false,
        }
    }
}

impl Hash for Identity {
    fn hash(self, ref mut state: Hasher) {
        match self {
            Identity::Address(address) => {
                0_u8.hash(state);
                address.hash(state);
            },
            Identity::ContractId(id) => {
                1_u8.hash(state);
                id.hash(state);
            },
            Identity::PredicateId(predicate_id) => {
                2_u8.hash(state);
                predicate_id.hash(state);
            },
        }
    }
}

#[test]
fn test_address() {
    let address = Address::from(ZERO_B256);
    let identity = Identity::Address(address);

    assert(identity.is_address());
    assert(!identity.is_contract_id());
    assert(!identity.is_predicate_id());

    assert(identity.as_address().unwrap() == address);
    assert(identity.as_contract_id().is_none());
    assert(identity.as_predicate_id().is_none());
}

#[test]
fn test_contract_id() {
    let id = ZERO_B256;
    let identity = Identity::ContractId(ContractId::from(ZERO_B256));

    assert(identity.is_contract_id());
    assert(!identity.is_address());
    assert(!identity.is_predicate_id());

    assert(identity.as_contract_id().unwrap().value == id);
    assert(identity.as_address().is_none());
    assert(identity.as_predicate_id().is_none());
}

#[test]
fn test_predicate() {
    let predicate_id = PredicateId::from(ZERO_B256);
    let identity = Identity::PredicateId(predicate_id);

    assert(identity.is_predicate_id());
    assert(!identity.is_address());
    assert(!identity.is_contract_id());

    assert(identity.as_predicate_id().unwrap() == predicate_id);
    assert(identity.as_address().is_none());
    assert(identity.as_contract_id().is_none());
}
