//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::From;
use ::hash::{Hash, Hasher};

/// The `ContractId` type, a struct wrapper around the inner `b256` value.
pub struct ContractId {
    /// The underlying raw `b256` data of the contract id.
    bits: b256,
}

impl ContractId {
    /// Returns the underlying raw `b256` data of the contract id.
    ///
    /// # Returns
    ///
    /// * [b256] - The raw data of the contract id.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() -> {
    ///     let my_contract = ContractId::from(ZERO_B256);
    ///     assert(my_contract.bits() == ZERO_B256);
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }
}

impl core::ops::Eq for ContractId {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
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
        Self { bits }
    }
}

impl From<ContractId> for b256 {
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
    fn from(id: ContractId) -> Self {
        id.bits()
    }
}

impl Hash for ContractId {
    fn hash(self, ref mut state: Hasher) {
        let Self { bits } = self;
        bits.hash(state);
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
    /// use std::{constants::ZERO_B256, asset::mint};
    ///
    /// fn foo() {
    ///     let this_contract = ContractId::this();
    ///     mint(ZERO_B256, 50);
    ///     Address::from(ZERO_B256).transfer(AssetId::default(this_contract), 50);
    /// }
    /// ```
    pub fn this() -> ContractId {
        ContractId::from(asm() {
            fp: b256
        })
    }
}
