//! The `Address` type used for interacting with addresses on the fuel network.
library;

use ::convert::{From, Into, TryFrom};
use ::hash::{Hash, Hasher};
use ::ops::*;
use ::primitives::*;
use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::codec::*;
use ::debug::*;

/// The `Address` type, a struct wrapper around the inner `b256` value.
pub struct Address {
    /// The underlying raw `b256` data of the address.
    bits: b256,
}

impl Address {
    /// Returns the underlying raw `b256` data of the address.
    ///
    /// # Returns
    ///
    /// * [b256] - The raw data of the address.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() -> {
    ///     let my_address = Address::zero();
    ///     assert(my_address.bits() == b256::zero());
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }

    /// Returns the zero value for the `Address` type.
    ///
    /// # Returns
    ///
    /// * [Address] -> The zero value for the `Address` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_address = Address::zero();
    ///     assert(zero_address == Address:from(b256::zero()));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bits: b256::zero(),
        }
    }

    /// Returns whether an `Address` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `Address` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_address = Address::zero();
    ///     assert(zero_address.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}

impl PartialEq for Address {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
    }
}
impl Eq for Address {}

/// Functions for casting between the `b256` and `Address` types.
impl From<b256> for Address {
    /// Casts raw `b256` data to an `Address`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [Address] - The newly created `Address` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///    let address = Address::from(b256::zero());
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { bits }
    }
}

impl From<Address> for b256 {
    /// Casts an `Address` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `Address`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let address = Address::zero();
    ///     let b256_data: b256 = b256::from(address);
    ///     assert(b256_data == b256::zero());
    /// }
    /// ```
    fn from(address: Address) -> Self {
        address.bits()
    }
}

impl TryFrom<Bytes> for Address {
    /// Casts raw `Bytes` data to an `Address`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The raw `Bytes` data to be casted.
    ///
    /// # Returns
    ///
    /// * [Address] - The newly created `Address` from the raw `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo(bytes: Bytes) {
    ///    let result = Address::try_from(bytes);
    ///    assert(result.is_some());
    ///    let address = result.unwrap();
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

impl Into<Bytes> for Address {
    /// Casts an `Address` to raw `Bytes` data.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The underlying raw `Bytes` data of the `Address`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let address = Address::zero();
    ///     let bytes_data: Bytes = address.into()
    ///     assert(bytes_data.len() == 32);
    /// }
    /// ```
    fn into(self) -> Bytes {
        Bytes::from(self.bits())
    }
}

impl Hash for Address {
    fn hash(self, ref mut state: Hasher) {
        self.bits.hash(state);
    }
}
