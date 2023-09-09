//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::alias::SubId;
use ::convert::From;
use ::hash::*;

/// The `ContractId` type, a struct wrapper around the inner `b256` value.
pub struct ContractId {
    /// The underlying raw `b256` data of the contract id.
    value: b256,
}

impl core::ops::Eq for ContractId {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

/// Functions for casting between the `b256` and `ContractId` types.
impl From<b256> for ContractId {
    /// Casts raw `b256` data to a `ContractId`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [ContractId] - The newly created `ContractId` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let contract_id = ContractId::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { value: bits }
    }

    /// Casts a `ContractId` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `ContractId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let contract_id = ContractId::from(ZERO_B256);
    ///     let b256_data = contract_id.into();
    ///     assert(b256_data == ZERO_B256);
    /// }
    /// ```
    fn into(self) -> b256 {
        self.value
    }
}

impl Hash for ContractId {
    fn hash(self, ref mut state: Hasher) {
        let Self { value } = self;
        value.hash(state);
    }
}

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
    value: b256,
}

impl Hash for AssetId {
    fn hash(self, ref mut state: Hasher) {
        let Self { value } = self;
        value.hash(state);
    }
}

impl core::ops::Eq for AssetId {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
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
        Self { value: bits }
    }

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
    fn into(self) -> b256 {
        self.value
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
        let value = sha256((contract_id, sub_id));
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
    ///     let asset_id = AssetId::default(contract_id);
    ///
    ///     assert(asset_id == AssetId::new(contract_id, sub_id));
    /// }
    /// ```
    pub fn default(contract_id: ContractId) -> Self {
        let value = sha256((
            contract_id,
            0x0000000000000000000000000000000000000000000000000000000000000000,
        ));
        Self { value }
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
    /// use std::{constants::ZERO_B256, token::transfer};
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
            value: 0x0000000000000000000000000000000000000000000000000000000000000000,
        }
    }
}

impl ContractId {
    /// Returns the ContractId of the currently executing contract.
    ///
    /// # Additional Information
    ///
    /// This is equivalent to std::callframes::contract_id().
    ///
    /// **_Note:_** If called in an external context, this will **not** return a ContractId.
    /// If called externally, will actually return a pointer to the Transaction Id (Wrapped in the ContractId struct).
    ///
    /// # Returns
    ///
    /// * [ContractId] - The contract id of this contract.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{constants::ZERO_B256, token::mint};
    ///
    /// fn foo() {
    ///     let this_contract = ContractId::this();
    ///     mint(ZERO_B256, 50);
    ///     Address::from(ZERO_B256).transfer(AssetId::default(this_contract), 50);
    /// }
    /// ```
    pub fn this() -> ContractId {
        ContractId::from(asm() { fp: b256 })
    }
    /// UNCONDITIONAL transfer of `amount` coins of type `asset_id` to
    /// the ContractId.
    ///
    /// # Additional Informations
    ///
    /// **_WARNING:_**
    /// This will transfer coins to a contract even with no way to retrieve them
    /// (i.e. no withdrawal functionality on receiving contract), possibly leading
    /// to the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// # Arguments
    ///
    /// * `asset_id`: [AssetId] - The `AssetId` of the token to transfer.
    /// * `amount`: [u64] - The amount of tokens to transfer.
    ///
    /// # Reverts
    ///
    /// * When `amount` is greater than the contract balance for `asset_id`.
    /// * When `amount` is equal to zero.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::{BASE_ASSET_ID, ZERO_B256};
    ///
    /// fn foo() {
    ///     let contract_id = ContractId::from(ZERO_B256);
    ///     contract_id.transfer(BASE_ASSET_ID, 500);
    /// }
    /// ```
    pub fn transfer(self, asset_id: AssetId, amount: u64) {
        asm(r1: amount, r2: asset_id.value, r3: self.value) {
            tr   r3 r1 r2;
        }
    }
}

impl ContractId {
    /// Mint `amount` coins of `sub_id` and send them  UNCONDITIONALLY to the contract at `to`.
    ///
    /// # Additional Information
    ///
    /// **_WARNING:_**
    /// This will transfer coins to a contract even with no way to retrieve them
    /// (i.e: no withdrawal functionality on the receiving contract), possibly leading to
    /// the **_PERMANENT LOSS OF COINS_** if not used with care.
    ///
    /// # Arguments
    ///
    /// * `sub_id`: [SubId] - The  sub identfier of the asset which to mint.
    /// * `amount`: [u64] - The amount of tokens to mint.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let contract_id = ContractId::from(ZERO_B256);
    ///     contract_id.mint_to(ZERO_B256, 500);
    /// }
    /// ```
    pub fn mint_to(self, sub_id: SubId, amount: u64) {
        asm(r1: amount, r2: sub_id) {
            mint r1 r2;
        };
        self.transfer(AssetId::new(ContractId::from(asm() { fp: b256 }), sub_id), amount);
    }
}

#[test()]
fn test_hasher_sha256_asset_id() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000).hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000001).hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}

#[test()]
fn test_hasher_sha256_contract_id() {
    use ::assert::assert;
    let mut hasher = Hasher::new();
    ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000).hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let mut hasher = Hasher::new();
    ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001).hash(hasher);
    let s256 = hasher.sha256();
    assert(s256 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);
}
