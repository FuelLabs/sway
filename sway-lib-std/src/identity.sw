//! A wrapper type with two variants, `Address` and `ContractId`.
//! The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library;

use ::assert::assert;
use ::address::Address;
use ::alias::SubId;
use ::call_frames::contract_id;
use ::constants::{BASE_ASSET_ID, ZERO_B256};
use ::contract_id::{AssetId, ContractId};
use ::hash::*;
use ::option::Option;

/// The `Identity` type: either an `Address` or a `ContractId`.
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
            Self::Address(addr) => Option::Some(addr),
            Self::ContractId(_) => Option::None,
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
            Self::Address(_) => Option::None,
            Self::ContractId(id) => Option::Some(id),
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

    /// Transfer `amount` coins of the type `asset_id` and send them
    /// to the Identity.
    ///
    /// # Additional Information
    ///
    /// **_WARNING:_**
    /// If the Identity is a contract this may transfer coins to the contract even with no way to retrieve them
    /// (i.e. no withdrawal functionality on receiving contract), possibly leading
    /// to the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// # Arguments
    ///
    /// * `asset_id`: [AssetId] - The `AssetId` of the token to transfer.
    /// * `amount`: [u64] - The amount of tokens to transfer.
    ///
    /// # Reverts
    ///
    /// * When `amount` is greater than the contract balance for `asset_id`.
    /// * When `amount` is equal to zero.
    /// * When there are no free variable outputs when transferring to an `Address`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// fn foo() {
    ///     let to_address = Identity::Address(Address::from(ZERO_B256));
    ///     let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
    ///     to_address.transfer(BASE_ASSET_ID, 500);
    ///     to_contract_id.transfer(BASE_ASSET_ID, 500);
    /// }
    /// ```
    pub fn transfer(self, asset_id: AssetId, amount: u64) {
        match self {
            Identity::Address(addr) => addr.transfer(asset_id, amount),
            Identity::ContractId(id) => id.transfer(asset_id, amount),
        };
    }
}

impl Identity {
    /// Mint `amount` coins of `sub_id` and transfer them to the Identity.
    ///
    /// # Additional Information
    ///
    /// **_WARNING:_**
    /// If the Identity is a contract, this will transfer coins to the contract even with no way to retrieve them
    /// (i.e: no withdrawal functionality on the receiving contract), possibly leading to
    /// the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// # Arguments
    ///
    /// * `sub_id`: [SubId] - The  sub identfier of the asset which to mint.
    /// * `amount`: [u64] - The amount of tokens to mint.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let address_identity = Identity::Address(Address::from(ZERO_B256));
    ///     let contract_identity = Identity::ContractId(ContractId::from(ZERO_B256));
    ///     address_identity.mint_to(ZERO_B256, 500);
    ///     contract_identity.mint_to(ZERO_B256, 500);
    /// }
    /// ```
    pub fn mint_to(self, sub_id: SubId, amount: u64) {
        asm(r1: amount, r2: sub_id) {
            mint r1 r2;
        };
        self.transfer(AssetId::new(contract_id(), sub_id), amount);
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
    assert(identity.as_contract_id().unwrap().value == id);
    assert(identity.as_address().is_none());
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
