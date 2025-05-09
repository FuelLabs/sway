library;

use ::convert::{From, TryFrom};
use ::bytes::{Bytes, *};
use ::option::Option::{self, *};
use ::ops::*;
use ::primitive_conversions::u256::*;
use ::codec::*;
use ::debug::*;

// NOTE: Bytes are used to support numbers greater than 32 bytes for future curves.
/// The Scalar type used in cryptographic operations.
pub struct Scalar {
    bytes: Bytes,
}

// All scalars must be of length 32
impl PartialEq for Scalar {
    fn eq(self, other: Self) -> bool {
        self.bytes.len() == 32 && self.bytes == other.bytes
    }
}
// Note that `Scalar` implements `PartialEq` but not `Eq`,
// because an uninitialized `Scalar`, created by `Scalar::new`
// is not equal to any other scalar, including itself.

impl Scalar {
    /// Returns a new, uninitialized Scalar.
    ///
    /// # Returns
    ///
    /// * [Scalar] - The new Scalar.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::scalar::Scalar;
    ///
    /// fn foo() {
    ///     let new_scalar = Scalar::new();
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Returns a zeroed Scalar.
    ///
    /// # Returns
    ///
    /// * [Scalar] - The new zeroed Scalar.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::scalar::Scalar;
    ///
    /// fn foo() {
    ///     let zero_scalar = Scalar::zero();
    ///     assert(b256::try_from(new_scalar.bytes()).unwrap() == b256::zero());
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bytes: Bytes::from(b256::zero()),
        }
    }

    /// Returns the minimum scalar.
    ///
    /// # Returns
    ///
    /// * [Scalar] - The new minimum Scalar.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::scalar::Scalar;
    ///
    /// fn foo() {
    ///     let zero_scalar = Scalar::zero();
    ///     assert(b256::try_from(new_scalar.bytes()).unwrap() == b256::zero());
    /// }
    /// ```
    pub fn min() -> Self {
        Self {
            bytes: Bytes::from(b256::zero()),
        }
    }

    /// Returns true if the scalar is zero, otherwise false.
    ///
    /// # Returns
    ///
    // * [bool] - The boolean representing whether the scalar is zero.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::scalar::Scalar;
    ///
    /// fn foo() {
    ///     let zero_scalar = Scalar::zero();
    ///     assert(zero_scalar.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        // Note that we could simply return `self == Self::zero()` here,
        // but this would cause creating a new `Scalar` zero instance
        // every time we call this function. `Self::zero()` is expensive
        // both in terms of gas and allocated memory.
        // In cases like calling this function in a loop, the performance
        // impact would be significant.
        self.bytes.len() == 32 && self.bytes.are_all_zero()
    }

    /// Returns the underlying bytes of the scalar.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The scalar represented as bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::scalar::Scalar;
    ///
    /// fn foo(scalar: Scalar) {
    ///     let bytes = scalar.bytes();
    ///     assert(bytes.len() != 0);
    /// }
    /// ```
    pub fn bytes(self) -> Bytes {
        self.bytes
    }
}

impl From<u256> for Scalar {
    fn from(bytes: u256) -> Self {
        Self {
            bytes: Bytes::from(bytes.as_b256()),
        }
    }
}

impl From<b256> for Scalar {
    fn from(bytes: b256) -> Self {
        Self {
            bytes: Bytes::from(bytes),
        }
    }
}

impl From<[u8; 32]> for Scalar {
    fn from(bytes_array: [u8; 32]) -> Self {
        // TODO: Once const generics are available directly call `From<[u8; N]>` on `Bytes`
        //       instead of having a loop.
        let mut bytes = Bytes::with_capacity(32);

        let mut iter = 0;
        while iter < 32 {
            bytes.push(bytes_array[iter]);
            iter += 1;
        }

        Self { bytes: bytes }
    }
}

impl TryFrom<Scalar> for u256 {
    fn try_from(scalar: Scalar) -> Option<Self> {
        if scalar.bytes.len() != 32 {
            return None;
        }

        let mut value = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
        let ptr = __addr_of(value);

        scalar.bytes.ptr().copy_to::<u256>(ptr, 1);

        Some(value)
    }
}

impl TryFrom<Scalar> for b256 {
    fn try_from(scalar: Scalar) -> Option<Self> {
        if scalar.bytes.len() != 32 {
            return None;
        }

        let mut value = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr = __addr_of(value);

        scalar.bytes.ptr().copy_to::<b256>(ptr, 1);

        Some(value)
    }
}
