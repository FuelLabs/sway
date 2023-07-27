//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::From;

/// The `ContractId` type, a struct wrapper around the inner `b256` value.
pub struct ContractId {
    /// The underlying raw `b256` data of the contract id.
    value: b256,
}

impl core::ops::Eq for ContractId {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `ContractId` types.
impl From<b256> for ContractId {
    /// Casts raw `b256` data to a `ContractId`.
    /// 
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    /// 
    /// # Returns
    ///
    /// * [ContractId] - The newly created `ContractId` from the raw `b256`.
    ///
    /// # Examples
    /// 
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let contract_id = ContractId::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> ContractId {
        ContractId { value: bits }
    }


    /// Casts a `ContractId` to raw `b256` data.
    /// 
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `ContractId`.
    ///
    /// # Examples
    /// 
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let contract_id = ContractId::from(ZERO_B256);
    ///     let b256_data = contract_id.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn into(self) -> b256 {
        self.value
    }
}

impl ContractId {
    /// UNCONDITIONAL transfer of `amount` coins of type `asset_id` to
    /// the ContractId.
    ///
    /// # Additional Informations
    ///
    /// **_WARNING:_**
    /// This will transfer coins to a contract even with no way to retrieve them
    /// (i.e. no withdrawal functionality on receiving contract), possibly leading
    /// to the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// # Arguments
    ///
    /// * `amount`: [u64] - The amount of tokens to transfer.
    /// * `asset_id`: [AssetId] - The `AssetId` of the token to transfer.
    ///
    /// # Panics
    ///
    /// * When `amount` is greater than the contract balance for `asset_id`.
    /// * When `amount` is equal to zero.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// fn foo() {
    ///     // replace the zero ContractId with your desired ContractId
    ///     let contract_id = ContractId::from(ZERO_B256);
    ///     contract_id.transfer(500, BASE_ASSET_ID);
    /// }
    /// ```
    pub fn transfer(self, amount: u64, asset_id: AssetId) {
        asm(r1: amount, r2: asset_id.value, r3: self.value) {
            tr r3 r1 r2;
        }
    }
}

impl ContractId {
    /// Mint `amount` coins of the current contract's `asset_id` and send them
    /// UNCONDITIONALLY to the contract at `to`.
    ///
    /// # Additional Information
    ///
    /// **_WARNING:_**
    /// This will transfer coins to a contract even with no way to retrieve them
    /// (i.e: no withdrawal functionality on the receiving contract), possibly leading to
    /// the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// # Arguments
    ///
    /// * `amount`: [u64] - The amount of tokens to mint.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     // replace the zero ContractId with your desired ContractId
    ///     let contract_id = ContractId::from(ZERO_B256);
    ///     contract_id.mint_to(500);
    /// }
    /// ```
    pub fn mint_to(self, amount: u64) {
        asm(r1: amount) {
            mint r1;
        };
        self.transfer(amount, ContractId::from(asm() { fp: b256 })); // Transfer the self contract token
    }
}

/// The `AssetId` type is simply an alias for `ContractId` that represents the ID of a native asset
/// which matches the ID of the contract that implements that asset.
pub type AssetId = ContractId;
