//! Functionality for accessing context-specific information about the current contract or message.
library context;
dep registers;
dep call_frames;

use ::contract_id::ContractId;
use ::call_frames::contract_id;
use ::registers::balance;

/// Get the balance of coin `asset_id` for the current contract.
pub fn this_balance(asset_id: ContractId) -> u64 {
    balance_of(asset_id, contract_id())
}

/// Get the balance of coin `asset_id` for for the contract at 'target'.
pub fn balance_of(asset_id: ContractId, target: ContractId) -> u64 {
    asm(balance, token: asset_id.value, id: target.value) {
        bal balance token id;
        balance: u64
    }
}

/// Get the remaining gas in the context.
pub fn gas() -> u64 {
    asm() { cgas: u64 }
}

/// Get the amount of units of `call_frames::msg_asset_id()` being sent.
pub fn msg_amount() -> u64 {
    balance()
}
