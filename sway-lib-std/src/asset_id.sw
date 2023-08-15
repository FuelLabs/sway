//! Helper functions for generating an AssetId
library;

use ::alias::{AssetId, SubId};
use ::constants::ZERO_B256;
use ::contract_id::ContractId;
use ::hash::sha256;

/// Construct an AssetId from a ContractId and SubId by hashing them together with sha256.
pub fn construct_asset_id(contract_id: ContractId, sub_id: SubId) -> AssetId {
    sha256((contract_id, sub_id))
}

/// Construct an AssetId from a ContractId using the default SubId (ZERO_B256).
pub fn construct_default_asset_id(contract_id: ContractId) -> AssetId {
    sha256((contract_id, ZERO_B256))
}
