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
            (Identity::Address(addr1), Identity::Address(addr2)) => addr1 == addr2,
            (Identity::ContractId(id1), Identity::ContractId(id2)) => id1 == id2,
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
            Self::ContractId(_) => None,
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
            Self::Address(_) => None,
            Self::ContractId(id) => Some(id),
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
            Self::ContractId(_) => false,
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
            Self::Address(_) => false,
            Self::ContractId(_) => true,
        }
    }

    /// Returns the underlying raw `b256` data of the identity.
    ///
    /// # Returns
    ///
    /// * [b256] - The raw data of the identity.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() -> {
    ///     let my_identity = Identity::Address(Address::from(ZERO_B256));
    ///     assert(my_identity.bits() == ZERO_B256);
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        match self {
            Self::Address(address) => address.bits(),
            Self::ContractId(contract_id) => contract_id.bits(),
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
}

#[test]
fn test_contract_id() {
    let id = ZERO_B256;
    let identity = Identity::ContractId(ContractId::from(ZERO_B256));
    assert(!identity.is_address());
    assert(identity.is_contract_id());
    assert(identity.as_contract_id().unwrap().bits() == id);
    assert(identity.as_address().is_none());
}
