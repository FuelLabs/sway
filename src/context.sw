library context;
//! Functionality for accessing context-specific information about the current contract or message.

use ::contract_id::ContractId;
use ::call_frames::*;

dep context/call_frames;
dep context/registers;

/// Get the current contracts balance of coin `asset_id`
pub fn this_balance(asset_id: ContractId) -> u64 {
    let this_id = contract_id();
    asm(balance, token: asset_id.value, contract_id: this_id.value) {
        bal balance token contract_id;
        balance: u64
    }
}

/// Get the balance of coin `asset_id` for any contract `contract_id`
pub fn balance_of_contract(asset_id: ContractId, ctr_id: ContractId) -> u64 {
    asm(balance, token: asset_id.value, contract: ctr_id.value) {
        bal balance token contract;
        balance: u64
    }
}

/// Get the amount of units of `msg_asset_id()` being sent.
pub fn msg_amount() -> u64 {
    asm() {
        bal: u64
    }
}

/// Get the remaining gas in the context.
pub fn gas() -> u64 {
    asm() {
        cgas: u64
    }
}

/// Get the remaining gas globally.
pub fn global_gas() -> u64 {
    asm() {
        ggas: u64
    }
}
