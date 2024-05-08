//! A wrapper around the `b256` type to help enhance type-safety.
library;

use ::convert::{Into, From};
use ::hash::{Hash, Hasher};
use ::math::Zero;

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
    /// fn foo() -> {
    ///     let my_contract = ContractId:zero();
    ///     assert(my_contract.bits() == b256::zero());
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
    /// fn foo() {
    ///    let contract_id = ContractId::from(b256::zero());
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { bits }
    }
}

impl Into<b256> for ContractId {
    /// Casts a `ContractId` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `ContractId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let contract_id = ContractId::zero();
    ///     let b256_data = contract_id.into();
    ///     assert(b256_data == b256::zero());
    /// }
    /// ```
    fn into(self) -> b256 {
        self.bits()
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
    /// use std::asset::mint;
    ///
    /// fn foo() {
    ///     let this_contract = ContractId::this();
    ///     mint(b256::zero(), 50);
    ///     Address::zero().transfer(AssetId::default(this_contract), 50);
    /// }
    /// ```
    pub fn this() -> ContractId {
        ContractId::from(asm() {
            fp: b256
        })
    }
}

impl Zero for ContractId {
    fn zero() -> Self {
        Self {
            bits: b256::zero()
        }
    }

    fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}

#[test]
fn test_into() {
    use ::assert::assert;
    
    let contract_id = ContractId::zero();
    let b256_data = contract_id.into();
    assert(b256_data == b256::zero());
}
