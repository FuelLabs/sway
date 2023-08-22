//! Base asset and zero address constants.
library;

use ::alias::SubId;
use ::contract_id::{AssetId, ContractId};

/// The `BASE_ASSET_ID` represents the base asset of a chain.
///
/// # Additional Information
///
/// On the Fuel network, the base asset is Ether. It is hardcoded as the 0x00..00 ContractId.
///
/// # Examples
/// 
/// ```sway
/// use std::{call_frames::msg_asset_id, constants::BASE_ASSET_ID};
///
/// fn foo() {
///     assert(BASE_ASSET_ID == msg_asset_id());
/// }
/// ```
pub const BASE_ASSET_ID: AssetId = AssetId {
    value: ZERO_B256,
};

/// A B256 of zero value.
///
/// # Examples
/// 
/// ```sway
/// use std::{call_frames::msg_asset_id, constants::ZERO_B256};
///
/// fn foo() {
///     assert(ZERO_B256 == msg_asset_id());
/// }
/// ```
pub const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

/// A SubId of zero value.
///
/// # Examples
///
/// ```sway
/// use std::{token::mint, constants::ZERO_SUB_ID};
///
/// fn foo() {
///     mint(ZERO_SUB_ID, 50); // Mint 50 tokens with a zero sub id.
/// }
/// ```
pub const ZERO_SUB_ID: SubId = SubId {
    value: ZERO_B256,
};