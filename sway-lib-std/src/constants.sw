//! Base asset and zero address constants.
library;

use ::contract_id::ContractId;

/// The `BASE_ASSET_ID` represents the base asset of a chain.
/// This is currently hard coded as a zero address, but will be configurable in the future.
pub const BASE_ASSET_ID = ContractId::from(ZERO_B256);
pub const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
