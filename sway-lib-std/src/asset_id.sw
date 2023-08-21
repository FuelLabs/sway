//! Helper functions for generating an AssetId
library;

use ::alias::{AssetId, SubId};
use ::constants::ZERO_B256;
use ::contract_id::ContractId;
use ::hash::*;

/// An AssetId is used for interacting with an asset on the network. 
///
/// # Additional Information
///
/// It is calculated by taking the sha256 hash of the originating ContractId and a SubId.
/// ie. sha256((contract_id, sub_id)).
///
/// The SubId is used to differentiate between different assets that are created by the same contract.
pub struct AssetId {
    value: b256,
}

impl AssetId {
    /// Represents bridged Ether on the main Fuel Network. Can be configured to represent another asset on another instance of the Fuel network.
    pub const BASE_ASSET_ID: AssetId = AssetId {
        /// sha256((ZERO_B256, ZERO_B256))). Prehashed to save gas.
        value: 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b,
    };

    /// Creates a new AssetId from a ContractId and SubId.
    ///
    /// # Arguments
    ///
    /// * `contract_id`: [ContractId] - The ContractId of the contract that created the asset.
    /// * `sub_id`: [SubId] - The SubId of the asset.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{callframes::contract_id, constants::ZERO_B256};
    ///
    /// fn foo() {
    ///     let contract_id = contract_id();
    ///     let sub_id = ZERO_B256;
    ///
    ///     let asset_id = AssetId::new(contract_id, sub_id);        
    /// }
    /// ```
    pub fn new(contract_id: ContractId, sub_id: SubId) -> Self {
        let value = sha256((contract_id, sub_id))
        Self { value }
    }

    /// Creates a new AssetId from a ContractId and the zero SubId.
    ///
    /// # Arguments
    ///
    /// * `contract_id`: [ContractId] - The ContractId of the contract that created the asset.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and the zero SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{callframes::contract_id, constants::ZERO_B256};
    ///
    /// fn foo() {
    ///     let contract_id = contract_id();
    ///     let sub_id = ZERO_B256;
    ///
    ///     let asset_id = AssetId::standard(contract_id);
    ///
    ///     assert(asset_id == AssetId::new(contract_id, sub_id));
    /// }
    /// ```
    pub fn standard(contract_id: ContractId) -> Self {
        let value = sha256((contract_id, ZERO_B256))
        Self { value }
    }

    /// Represents bridged Ether on the main Fuel Network. Can be configured to represent another asset on another instance of the Fuel network.
    ///
    /// # Additional Information
    ///
    /// The base asset id is minted from the zero ContractId with the zero SubId.
    /// It is computed as sha256((ZERO_B256, ZERO_B256))).
    ///
    /// NOT equal to ZERO_B256.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the zero ContractId and the zero SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::token::transfer;
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::base_asset_id();
    ///     let amount = 100;
    ///     let recipient = Identity::ContractId(ContractId::from(ZERO_B256));
    ///
    ///     transfer(recipient, asset_id, amount);
    /// ```
    pub fn base_asset_id() -> Self {
        Self {
            /// sha256((ZERO_B256, ZERO_B256))). Prehashed to save gas.
            value: 0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b,
        }

    }
}