//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::alias::SubId;
use ::convert::From;
use ::hash::*;

/// The `ContractId` type, a struct wrapper around the inner `b256` value.
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
    fn from(bits: b256) -> Self {
        Self { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}

impl ContractId {
    /// UNCONDITIONAL transfer of `amount` coins of type `asset_id` to
    /// the ContractId.
    ///
    /// > **_WARNING:_**
    /// >
    /// > This will transfer coins to a contract even with no way to retrieve them
    /// > (i.e. no withdrawal functionality on receiving contract), possibly leading
    /// > to the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// ### Arguments
    ///
    /// * `asset_id` - The `AssetId` of the token to transfer.
    /// * `amount` - The amount of tokens to transfer.
    ///
    /// ### Reverts
    ///
    /// * If `amount` is greater than the contract balance for `asset_id`.
    /// * If `amount` is equal to zero.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// // replace the zero ContractId with your desired ContractId
    /// let contract_id = ContractId::from(ZERO_B256);
    /// contract_id.transfer(BASE_ASSET_ID, 500);
    /// ```
    pub fn transfer(self, asset_id: AssetId, amount: u64) {
        asm(r1: amount, r2: asset_id.value, r3: self.value) {
            tr r3 r1 r2;
        }
    }
}

impl Hash for ContractId {
    fn hash(self, ref mut state: Hasher) {
        let ContractId { value } = self;
        value.hash(state);
    }
}

impl ContractId {
    /// Mint `amount` coins of `sub_id` and send them  UNCONDITIONALLY to the contract at `to`.
    ///
    /// > **_WARNING:_**
    /// >
    /// > This will transfer coins to a contract even with no way to retrieve them
    /// > (i.e: no withdrawal functionality on the receiving contract), possibly leading to
    /// > the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// ### Arguments
    ///
    /// * `sub_id` - The  sub identfier of the asset which to mint.
    /// * `amount` - The amount of tokens to mint.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// // replace the zero ContractId with your desired ContractId
    /// let contract_id = ContractId::from(ZERO_B256);
    /// contract_id.mint_to(ZERO_B256, 500);
    /// ```
    pub fn mint_to(self, sub_id: SubId, amount: u64) {
        asm(r1: amount, r2: sub_id) {
            mint r1 r2;
        };
        self.transfer(AssetId::new(ContractId::from(asm() { fp: b256 }), sub_id), amount);
    }
}

/// An AssetId is used for interacting with an asset on the network. 
///
/// # Additional Information
///
/// It is calculated by taking the sha256 hash of the originating ContractId and a SubId.
/// ie. sha256((contract_id, sub_id)).
///
/// An exception is the Base Asset, which is just the ZERO_B256 AssetId.
///
/// The SubId is used to differentiate between different assets that are created by the same contract.
pub struct AssetId {
    value: b256,
}

impl AssetId {
    /// Creates a new AssetId from a ContractId and SubId.
    ///
    /// # Arguments
    ///
    /// * `contract_id`: [ContractId] - The ContractId of the contract that created the asset.
    /// * `sub_id`: [SubId] - The SubId of the asset.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{callframes::contract_id, constants::ZERO_B256};
    ///
    /// fn foo() {
    ///     let contract_id = contract_id();
    ///     let sub_id = ZERO_B256;
    ///
    ///     let asset_id = AssetId::new(contract_id, sub_id);        
    /// }
    /// ```
    pub fn new(contract_id: ContractId, sub_id: SubId) -> Self {
        let value = sha256((contract_id, sub_id));
        Self { value }
    }

    /// Creates a new AssetId from a ContractId and the zero SubId.
    ///
    /// # Arguments
    ///
    /// * `contract_id`: [ContractId] - The ContractId of the contract that created the asset.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and the zero SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{callframes::contract_id, constants::ZERO_B256};
    ///
    /// fn foo() {
    ///     let contract_id = contract_id();
    ///     let sub_id = ZERO_B256;
    ///
    ///     let asset_id = AssetId::standard(contract_id);
    ///
    ///     assert(asset_id == AssetId::new(contract_id, sub_id));
    /// }
    /// ```
    pub fn standard(contract_id: ContractId) -> Self {
        let value = sha256((contract_id, 0x0000000000000000000000000000000000000000000000000000000000000000));
        Self { value }
    }

    /// Represents bridged Ether on the main Fuel Network. Can be configured to represent another asset on another instance of the Fuel network.
    ///
    /// # Additional Information
    ///
    /// It is hard coded to be ZERO_B256.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the zero ContractId and the zero SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::token::transfer;
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::base_asset_id();
    ///     let amount = 100;
    ///     let recipient = Identity::ContractId(ContractId::from(ZERO_B256));
    ///
    ///     transfer(recipient, asset_id, amount);
    /// ```
    pub fn base_asset_id() -> Self {
        Self {
            value: 0x0000000000000000000000000000000000000000000000000000000000000000,
        }
    }
}

