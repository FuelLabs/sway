//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::call_frames::contract_id;
use ::contract_id::{AssetId};
use ::convert::From;

/// The `Address` type, a struct wrapper around the inner `b256` value.
pub struct Address {
    value: b256,
}

impl core::ops::Eq for Address {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `Address` types.
impl From<b256> for Address {
    fn from(bits: b256) -> Address {
        Address { value: bits }
    }

    fn into(self) -> b256 {
        self.value
    }
}

impl Address {
    /// Transfer `amount` coins of type `asset_id` and send them to
    /// the Address.
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
    /// * If there are no free variable outputs.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// // replace the zero Address with your desired Address
    /// let address = Address::from(ZERO_B256);
    /// address.transfer(500, BASE_ASSET_ID)
    /// ```
    pub fn transfer(self, amount: u64, asset_id: AssetId) {
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
                        tro r1 r2 r3 r4;
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
    /// ### Arguments
    ///
    /// * `amount` - The amount of tokens to mint.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// // replace the zero Address with your desired Address
    /// let address = Address::from(ZERO_B256);
    /// address.mint_to(500);
    /// ```
    pub fn mint_to(self, amount: u64) {
        asm(r1: amount) {
            mint r1;
        };
        self.transfer(amount, contract_id());
    }
}
