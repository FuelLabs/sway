//! Base asset and zero address constants.
library;

use ::contract_id::ContractId;

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
///     // Ensures that assets being transferred are the base asset.
///     assert(BASE_ASSET_ID == msg_asset_id());
/// }
/// ```
pub const BASE_ASSET_ID = ContractId::from(ZERO_B256);

/// A B256 of zero value.
///
/// # Examples
/// 
/// ```sway
/// use std::{call_frames::msg_asset_id, constants::ZERO_B256};
///
/// fn foo() {
///     let base_asset_id = ContractId::from(ZERO_B256);
///     
///     // Ensures that assets being transferred are the base asset.
///     assert(base_asset_id == msg_asset_id());
/// }
/// ```
pub const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
