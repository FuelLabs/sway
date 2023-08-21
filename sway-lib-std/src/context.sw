//! Functionality for accessing context-specific information about the current contract or message.
library;

use ::call_frames::contract_id;
use ::contract_id::{AssetId, ContractId};
use ::registers::balance;

/// Get the balance of coin `asset_id` for the current contract.
pub fn this_balance(asset_id: AssetId) -> u64 {
    balance_of(contract_id(), asset_id)
}

/// Get the balance of coin `asset_id` for the contract at 'target'.
pub fn balance_of(target: ContractId, asset_id: AssetId) -> u64 {
    asm(balance, token: asset_id.value, id: target.value) {
        bal balance token id;
        balance: u64
    }
}

/// Get the amount of units of `call_frames::msg_asset_id()` being sent.
pub fn msg_amount() -> u64 {
    balance()
}
