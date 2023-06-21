//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::alias::{AssetId, SubId};
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
        asm(r1: amount, r2: asset_id, r3: self.value) {
            tr r3 r1 r2;
        }
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
        self.transfer(sha256((ContractId::from(asm() { fp: b256 }), sub_id)), amount);
    }
}

impl Hash for ContractId {
    fn hash(self, ref mut state: Hasher) {
        self.value.hash(state);
    }
}