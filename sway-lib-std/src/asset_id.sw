//! The `AssetId` type used for interacting with an asset on the fuel network.
library;

use ::alias::SubId;
use ::contract_id::ContractId;
use ::convert::{From, Into, TryFrom};
use ::hash::{Hash, Hasher};
use ::ops::*;
use ::primitives::*;
use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::codec::*;
/// An AssetId is used for interacting with an asset on the network.
///
/// # Additional Information
///
/// It is calculated by taking the sha256 hash of the originating ContractId and a SubId.
/// i.e. sha256((contract_id, sub_id)).
///
/// An exception is the Base Asset.
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

impl PartialEq for AssetId {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
    }
}
impl Eq for AssetId {}

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
    /// fn foo() {
    ///    let asset_id = AssetId::from(b256::zero());
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
    /// use std::call_frames::contract_id;
    ///
    /// fn foo() {
    ///     let contract_id = contract_id();
    ///     let sub_id = b256::zero();
    ///
    ///     let asset_id = AssetId::new(contract_id, sub_id);
    /// }
    /// ```
    pub fn new(contract_id: ContractId, sub_id: SubId) -> Self {
        let result_buffer = b256::zero();
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
    /// # Additional Information
    ///
    /// **WARNING** If called in an external context, this will **not** return a correct AssetId.
    /// If called externally, will actually use the Transaction Id as the ContractId.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and the default SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{call_frames::contract_id, constants::DEFAULT_SUB_ID};
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
        let result_buffer = b256::zero();
        asm(
            asset_id: result_buffer,
            ptr: (contract_id, b256::zero()),
            bytes: 64,
        ) {
            s256 asset_id ptr bytes;
        };

        Self {
            bits: result_buffer,
        }
    }

    /// The base asset of a chain.
    ///
    /// # Additional Information
    ///
    /// On the Fuel network, the base asset is Ether.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the base asset.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::asset::transfer;
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::base();
    ///     let amount = 100;
    ///     let recipient = Identity::ContractId(ContractId::zero());
    ///
    ///     transfer(recipient, asset_id, amount);
    /// ```
    pub fn base() -> Self {
        Self {
            bits: asm(r1) {
                gm r1 i6;
                r1: b256
            },
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
    /// fn foo() -> {
    ///     let my_asset = AssetId::from(b256::zero());
    ///     assert(my_asset.bits() == b256::zero());
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }

    /// Returns the zero value for the `AssetId` type.
    ///
    /// # Returns
    ///
    /// * [AssetId] -> The zero value for the `AssetId` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_asset_id = AssetId::zero();
    ///     assert(zero_asset_id == AssetId::from(b256::zero()));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bits: b256::zero(),
        }
    }

    /// Returns whether an `AssetId` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `AssetId` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_asset_id = AssetId::zero();
    ///     assert(zero_asset_id.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.bits == b256::zero()
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
    /// fn foo() {
    ///     let asset_id = AssetId::zero();
    ///     let b256_data: b256 = b256::from(asset_id);
    ///     assert(b256_data == b256::zero());
    /// }
    /// ```
    fn from(id: AssetId) -> Self {
        id.bits()
    }
}

impl TryFrom<Bytes> for AssetId {
    /// Casts raw `Bytes` data to an `AssetId`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The raw `Bytes` data to be casted.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The newly created `AssetId` from the raw `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo(bytes: Bytes) {
    ///    let result = AssetId::try_from(bytes);
    ///    assert(result.is_some());
    ///    let asset_id = result.unwrap();
    /// }
    /// ```
    fn try_from(bytes: Bytes) -> Option<Self> {
        if bytes.len() != 32 {
            return None;
        }

        Some(Self {
            bits: asm(ptr: bytes.ptr()) {
                ptr: b256
            },
        })
    }
}

impl Into<Bytes> for AssetId {
    /// Casts an `AssetId` to raw `Bytes` data.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The underlying raw `Bytes` data of the `AssetId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let asset_id = AssetId::zero();
    ///     let bytes_data: Bytes = asset_id.into();
    ///     assert(bytes_data.len() == 32);
    /// }
    /// ```
    fn into(self) -> Bytes {
        Bytes::from(self.bits())
    }
}
