//! Functionality for accessing context-specific information about the current contract or message.
library;

use ::contract_id::ContractId;
use ::call_frames::contract_id;
use ::registers::balance;

/// Get the balance of coin `asset_id` for the current contract.
///
/// # Arguments
///
/// * `asset_id`: [ContractId] - The asset of which the balance should be returned.
///
/// # Returns
///
/// * [u64] - The amount of the asset which the contract holds.
///
/// # Examples
///
/// ```sway
/// use std::{context::this_balance, token::mint, call_frames::contract_id};
/// 
/// fn foo() {
///     mint(50);
///     assert(this_balance(contract_id()) == 50);
/// }
/// ```
pub fn this_balance(asset_id: ContractId) -> u64 {
    balance_of(asset_id, contract_id())
}

/// Get the balance of coin `asset_id` for the contract at 'target'.
///
/// # Arguments
///
/// * `asset_id`: [ContractId] - The asset of which the balance should be returned.
/// * `target`: [ContractId] - The contract of which the balance should be returned.
///
/// # Returns
///
/// * [u64] - The amount of the asset which the `target` holds.
///
/// # Examples
///
/// ```sway
/// use std::{context::balance_of, token::mint, call_frames::contract_id};
/// 
/// fn foo() {
///     mint(50);
///     assert(balance_of(contract_id(), contract_id()) == 50);
/// }
/// ```
pub fn balance_of(asset_id: ContractId, target: ContractId) -> u64 {
    asm(balance, token: asset_id.value, id: target.value) {
        bal balance token id;
        balance: u64
    }
}

/// Get the amount of units of `call_frames::msg_asset_id()` being sent.
///
/// # Returns
///
/// * [u64] - The amount of tokens being sent.
///
/// # Examples 
/// 
/// ```sway
/// use std::context::msg_amount;
/// 
/// fn foo() {
///     // Ensures that no assets are being transferred in this context
///     assert(msg_amount() == 0);
/// }
/// ```
pub fn msg_amount() -> u64 {
    balance()
}
