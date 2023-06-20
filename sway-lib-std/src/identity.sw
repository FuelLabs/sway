//! A wrapper type with two variants, `Address` and `ContractId`.
//! The use of this type allows for handling interactions with contracts and addresses in a unified manner.
library;

use ::assert::assert;
use ::address::Address;
use ::constants::{ZERO_B256, BASE_ASSET_ID};
use ::contract_id::{AssetId, ContractId};
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
  
    /// Transfer `amount` coins of the type `asset_id` and send them
    /// to the Identity.
    ///
    /// > **_WARNING:_**
    /// >
    /// > If the Identity is a contract this may transfer coins to the contract even with no way to retrieve them
    /// > (i.e. no withdrawal functionality on receiving contract), possibly leading
    /// > to the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// ### Arguments
    ///
    /// * `amount` - The amount of tokens to transfer.
    /// * `asset_id` - The `AssetId` of the token to transfer.
    ///
    /// ### Reverts
    ///
    /// * If `amount` is greater than the contract balance for `asset_id`.
    /// * If `amount` is equal to zero.
    /// * If there are no free variable outputs when transferring to an `Address`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// // replace the zero Address/ContractId with your desired Address/ContractId
    /// let to_address = Identity::Address(Address::from(ZERO_B256));
    /// let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
    /// to_address.transfer(500, BASE_ASSET_ID);
    /// to_contract_id.transfer(500, BASE_ASSET_ID);
    /// ```
    pub fn transfer(self, amount: u64, asset_id: AssetId) {
        match to {
            Identity::Address(addr) => addr.transfer(amount, asset_id),
            Identity::ContractId(id) => id.transfer(amount, asset_id),
        };
    }
}

impl Identity {
    /// Mint `amount` coins of the current contract's `asset_id` and transfer them
    /// to the Identity.
    ///
    /// > **_WARNING:_**
    /// >
    /// > If the Identity is a contract, this will transfer coins to the contract even with no way to retrieve them
    /// > (i.e: no withdrawal functionality on the receiving contract), possibly leading to
    /// > the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// ### Arguments
    ///
    /// * `amount` - The amount of tokens to mint.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// // replace the zero Address/ContractId with your desired Address/ContractId
    /// let address_identity = Identity::Address(Address::from(ZERO_B256));
    /// let contract_identity = Identity::ContractId(ContractId::from(ZERO_B256));
    /// address_identity.mint_to(500);
    /// contract_identity.mint_to(500);
    /// ```
    pub fn mint_to(self, amount: u64) {
        asm(r1: amount) {
            mint r1;
        };
        self.transfer(amount, contract_id());
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
    let contract_id = BASE_ASSET_ID;
    let identity = Identity::ContractId(contract_id);
    assert(!identity.is_address());
    assert(identity.is_contract_id());
    assert(identity.as_contract_id().unwrap() == contract_id);
    assert(identity.as_address().is_none());
}
