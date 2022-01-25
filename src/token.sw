library token;
//! Functionality for performing common operations on tokens.

use ::contract_id::ContractId;

/// Mint `amount` coins of the current contract's `asset_id`.
pub fn mint(amount: u64) {
    asm(r1: amount) {
        mint r1;
    }
}

/// Burn `amount` coins of the current contract's `asset_id`.
pub fn burn(amount: u64) {
    asm(r1: amount) {
        burn r1;
    }
}

/// !!! UNCONDITIONAL transfer of `amount` coins of type `asset_id` to contract at `contract_id`.
/// This will allow the transfer of coins even if there is no way to retrieve them !!!
/// Use of this function can lead to irretrievable loss of coins if not used with caution.
pub fn force_transfer(amount: u64, asset_id: ContractId, contract_id: ContractId) {
    asm(r1: amount, r2: asset_id.value, r3: contract_id.value) {
        tr r3 r1 r2;
    }
}
