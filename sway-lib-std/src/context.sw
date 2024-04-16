//! Functionality for accessing context-specific information about the current contract or message.
library;

use ::asset_id::AssetId;
use ::contract_id::ContractId;
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
/// use std::{context::this_balance, constants::DEFAULT_SUB_ID, asset::mint};
///
/// fn foo() {
///     mint(DEFAULT_SUB_ID, 50);
///     let asset_id = AssetId::default();
///     assert(this_balance(asset_id)) == 50);
/// }
/// ```
pub fn this_balance(asset_id: AssetId) -> u64 {
    balance_of(ContractId::this(), asset_id)
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
/// use std::{asset::mint, call_frames::contract_id, constants::DEFAULT_SUB_ID, context::balance_of};
///
/// fn foo() {
///     mint(DEFAULT_SUB_ID, 50);
///     let asset_id = AssetId::default();
///     assert(balance_of(contract_id(), asset_id) == 50);
/// }
/// ```
pub fn balance_of(target: ContractId, asset_id: AssetId) -> u64 {
    asm(balance, asset: asset_id.bits(), id: target.bits()) {
        bal balance asset id;
        balance: u64
    }
}

/// Get the amount of units of `call_frames::msg_asset_id()` being sent.
///
/// # Returns
///
/// * [u64] - The amount of coins being sent.
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
