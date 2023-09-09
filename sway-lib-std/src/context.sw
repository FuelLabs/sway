//! Functionality for accessing context-specific information about the current contract or message.
library;

use ::call_frames::contract_id;
use ::contract_id::{AssetId, ContractId};
use ::registers::balance;

/// Get the balance of coin `asset_id` for the current contract.
///
/// # Arguments
///
/// * `asset_id`: [AssetId] - The asset of which the balance should be returned.
///
/// # Returns
///
/// * [u64] - The amount of the asset which the contract holds.
///
/// # Examples
///
/// ```sway
/// use std::{context::this_balance, constants::ZERO_B256, hash::sha256, token::mint, call_frames::contract_id};
///
/// fn foo() {
///     mint(ZERO_B256, 50);
///     assert(this_balance(sha256((ZERO_B256, contract_id()))) == 50);
/// }
/// ```
pub fn this_balance(asset_id: AssetId) -> u64 {
    balance_of(contract_id(), asset_id)
}

/// Get the balance of coin `asset_id` for the contract at 'target'.
///
/// # Arguments
///
/// * `target`: [ContractId] - The contract that contains the `asset_id`.
/// * `asset_id`: [AssetId] - The asset of which the balance should be returned.
///
/// # Returns
///
/// * [u64] - The amount of the asset which the `target` holds.
///
/// # Examples
///
/// ```sway
/// use std::{context::balance_of, constants::ZERO_B256, hash::sha256, token::mint, call_frames::contract_id};
///
/// fn foo() {
///     mint(ZERO_B256, 50);
///     assert(balance_of(contract_id(), sha256((ZERO_B256, contract_id()))) == 50);
/// }
/// ```
pub fn balance_of(target: ContractId, asset_id: AssetId) -> u64 {
    asm(balance, token: asset_id.value, id: target.value) {
        bal  balance token id;
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
///     assert(msg_amount() == 0);
/// }
/// ```
pub fn msg_amount() -> u64 {
    balance()
}
