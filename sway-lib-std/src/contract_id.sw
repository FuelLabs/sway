//! The `ContractId` type used for interacting with contracts on the fuel network.
library;

use ::convert::{From, Into, TryFrom};
use ::hash::{Hash, Hasher};
use ::ops::*;
use ::primitives::*;
use ::codec::*;
use ::debug::*;
use ::bytes::Bytes;
use ::option::Option::{self, *};

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

impl PartialEq for ContractId {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
    }
}
impl Eq for ContractId {}

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
    /// fn foo() {
    ///     let contract_id = ContractId::from(b256::zero());
    ///     let b256_data: b256 = contract_id.into();
    ///     assert(b256_data == b256::zero());
    /// }
    /// ```
    fn from(id: ContractId) -> Self {
        id.bits()
    }
}

impl TryFrom<Bytes> for ContractId {
    /// Casts raw `Bytes` data to an `ContractId`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The raw `Bytes` data to be casted.
    ///
    /// # Returns
    ///
    /// * [ContractId] - The newly created `ContractId` from the raw `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo(bytes: Bytes) {
    ///    let result = ContractId::try_from(bytes);
    ///    assert(result.is_some());
    ///    let contract_id = result.unwrap();
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

impl Into<Bytes> for ContractId {
    /// Casts an `ContractId` to raw `Bytes` data.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The underlying raw `Bytes` data of the `ContractId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let contract_id = ContractId::zero();
    ///     let bytes_data: Bytes = contract_id.into()
    ///     assert(bytes_data.len() == 32);
    /// }
    /// ```
    fn into(self) -> Bytes {
        Bytes::from(self.bits())
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

    /// Returns the zero value for the `ContractId` type.
    ///
    /// # Returns
    ///
    /// * [ContractId] -> The zero value for the `ContractId` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_contract_id = ContractId::zero();
    ///     assert(zero_contract_id == ContractId::from(b256::zero()));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bits: b256::zero(),
        }
    }

    /// Returns whether a `ContractId` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `ContractId` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_contract_id = ContractId::zero();
    ///     assert(zero_contract_id.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}
