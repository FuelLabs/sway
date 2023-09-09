//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::alias::SubId;
use ::call_frames::contract_id;
use ::contract_id::AssetId;
use ::convert::From;
use ::hash::*;
use ::error_signals::FAILED_TRANSFER_TO_ADDRESS_SIGNAL;
use ::hash::sha256;
use ::revert::revert;
use ::outputs::{Output, output_amount, output_count, output_type};

/// The `Address` type, a struct wrapper around the inner `b256` value.
pub struct Address {
    /// The underlying raw `b256` data of the address.
    value: b256,
}

impl core::ops::Eq for Address {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `Address` types.
impl From<b256> for Address {
    /// Casts raw `b256` data to an `Address`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [Address] - The newly created `Address` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let address = Address::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { value: bits }
    }

    /// Casts an `Address` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `Address`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let address = Address::from(ZERO_B256);
    ///     let b256_data = address.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn into(self) -> b256 {
        self.value
    }
}

impl Address {
    /// Transfer `amount` coins of type `asset_id` and send them to
    /// the Address.
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
    /// * When there are no free variable outputs.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// fn foo() {
    ///     let address = Address::from(ZERO_B256);
    ///     address.transfer(BASE_ASSET_ID, 500);
    /// }
    /// ```
    pub fn transfer(self, asset_id: AssetId, amount: u64) {
        // maintain a manual index as we only have `while` loops in sway atm:
        let mut index = 0;

        // If an output of type `OutputVariable` is found, check if its `amount` is
        // zero. As one cannot transfer zero coins to an output without a panic, a
        // variable output with a value of zero is by definition unused.
        let number_of_outputs = output_count();
        while index < number_of_outputs {
            if let Output::Variable = output_type(index) {
                if output_amount(index) == 0 {
                    asm(r1: self.value, r2: index, r3: amount, r4: asset_id.value) {
                        tro  r1 r2 r3 r4;
                    };
                    return;
                }
            }
            index += 1;
        }

        revert(FAILED_TRANSFER_TO_ADDRESS_SIGNAL);
    }
}

impl Address {
    /// Mint `amount` coins of the current contract's `asset_id` and send them to
    /// the Address.
    ///
    /// # Arguments
    ///
    /// * `sub_id`: [SubId] - The sub id of the token to mint.
    /// * `amount`: [u64] - The amount of tokens to mint.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let address = Address::from(ZERO_B256);
    ///     address.mint_to(ZERO_B256, 500);
    /// }
    /// ```
    pub fn mint_to(self, sub_id: SubId, amount: u64) {
        asm(r1: amount, r2: sub_id) {
            mint r1 r2;
        };
        self.transfer(AssetId::new(contract_id(), sub_id), amount);
    }
}

impl Hash for Address {
    fn hash(self, ref mut state: Hasher) {
        let Address { value } = self;
        value.hash(state);
    }
}
