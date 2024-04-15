//! Base asset and zero address constants.
library;

/// A b256 of zero value.
///
/// # Examples
///
/// ```sway
/// use std::{call_frames::msg_asset_id, constants::ZERO_B256};
///
/// fn foo() {
///     assert(ZERO_B256 == msg_asset_id().bits());
/// }
/// ```
pub const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

/// A u256 of zero value.
///
/// # Examples
///
/// ```sway
/// use std::constants::ZERO_U256;
///
/// fn foo() {
///     assert(ZERO_U256 == u256::from(0_u64));
/// }
/// ```
pub const ZERO_U256 = 0x00u256;

/// The default Sub Id for assets.
///
/// # Examples
///
/// ```sway
/// use std::{call_frames::contract_id, constants::DEFAULT_SUB_ID};
///
/// fn foo() {
///     let asset = AssetId::default();
///     assert(AssetId::new(contract_id(), DEFAULT_SUB_ID) == msg_asset_id());
/// }
/// ```
pub const DEFAULT_SUB_ID = ZERO_B256;
