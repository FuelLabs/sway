library;

use ::alias::SubId;
use ::contract_id::ContractId;
use ::convert::From;
use ::hash::{Hash, Hasher};

/// An AssetId is used for interacting with an asset on the network.
///
/// # Additional Information
///
/// It is calculated by taking the sha256 hash of the originating ContractId and a SubId.
/// i.e. sha256((contract_id, sub_id)).
///
/// An exception is the Base Asset, which is just the ZERO_B256 AssetId.
///
/// The SubId is used to differentiate between different assets that are created by the same contract.
pub struct AssetId {
    bits: b256,
}

impl Hash for AssetId {
    fn hash(self, ref mut state: Hasher) {
        let Self { bits } = self;
        bits.hash(state);
    }
}

impl core::ops::Eq for AssetId {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
    }
}

impl From<b256> for AssetId {
    /// Casts raw `b256` data to an `AssetId`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The newly created `AssetId` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let asset_id = AssetId::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { bits }
    }
}

impl AssetId {
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
        let result_buffer = 0x0000000000000000000000000000000000000000000000000000000000000000;
        asm(
            asset_id: result_buffer,
            ptr: (contract_id, sub_id),
            bytes: 64,
        ) {
            s256 asset_id ptr bytes;
        };

        Self {
            bits: result_buffer,
        }
    }

    /// Creates a new AssetId with the default SubId for the current contract.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and the default SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{callframes::contract_id, constants::DEFAULT_SUB_ID};
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::default();
    ///     assert(asset_id == AssetId::new(contract_id(), DEFAULT_SUB_ID));
    /// }
    /// ```
    pub fn default() -> Self {
        let contract_id = asm() {
            fp: b256
        };
        let result_buffer = 0x0000000000000000000000000000000000000000000000000000000000000000;
        asm(
            asset_id: result_buffer,
            ptr: (
                contract_id,
                0x0000000000000000000000000000000000000000000000000000000000000000,
            ),
            bytes: 64,
        ) {
            s256 asset_id ptr bytes;
        };

        Self {
            bits: result_buffer,
        }
    }

    /// The base_asset_id represents the base asset of a chain.
    ///
    /// # Additional Information
    ///
    /// On the Fuel network, the base asset is Ether. It is hardcoded as the 0x00..00 AssetId.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the base asset.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{constants::ZERO_B256, asset::transfer};
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
            bits: 0x0000000000000000000000000000000000000000000000000000000000000000,
        }
    }

    /// Returns the underlying raw `b256` data of the asset id.
    ///
    /// # Returns
    ///
    /// * [b256] - The raw data of the asset id.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() -> {
    ///     let my_asset = AssetId::from(ZERO_B256);
    ///     assert(my_asset.bits() == ZERO_B256);
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }
}

impl From<AssetId> for b256 {
    /// Casts an `AssetId` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `AssetId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::from(ZERO_B256);
    ///     let b256_data = asset_id.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn from(id: AssetId) -> Self {
        id.bits()
    }
}

#[test()]
fn test_hasher_sha256_asset_id() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000)
        .hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001)
        .hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test()]
fn test_hasher_sha256_contract_id() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000)
        .hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001)
        .hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}
