//! The `B512` type supports the usage of 64-byte values in Sway which are needed when working with public keys and signatures.
library;

use ::ops::*;
use ::primitives::*;
use ::convert::{From, Into, TryFrom};
use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::raw_slice::*;
use ::codec::*;
use ::debug::*;
use ::hash::*;

/// Stores two `b256`s in contiguous memory.
/// Guaranteed to be contiguous for use with ec-recover: `std::ecr::ec_recover`.
pub struct B512 {
    /// The two `b256`s that make up the `B512`.
    bits: [b256; 2],
}

impl PartialEq for B512 {
    fn eq(self, other: Self) -> bool {
        (self.bits)[0] == (other.bits)[0] && (self.bits)[1] == (other.bits)[1]
    }
}
impl Eq for B512 {}

impl From<(b256, b256)> for B512 {
    /// Converts from a `b256` tuple to a `B512`.
    ///
    /// # Arguments
    ///
    /// * `components`: [(b256, b256)] - The `(b256, b256)` tuple to convert to a `B512`.
    ///
    /// # Returns
    ///
    /// * [B512] - The newly created `B512`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let tuple: (b256, b256) = (b256::zero(), b256::zero());
    ///     let b512 = B512::from(tuple);
    /// }
    /// ```
    fn from(components: (b256, b256)) -> Self {
        Self {
            bits: [components.0, components.1],
        }
    }
}

impl From<B512> for (b256, b256) {
    /// Converts from a `B512` to a `b256` tuple.
    ///
    /// # Additional Information
    ///
    /// **NOTE:** To import, use the glob operator i.e. `use std::b512::*;`
    ///
    /// # Arguments
    ///
    /// * `val`: [B512] - The `B512` to convert to a tuple.
    ///
    /// # Returns
    ///
    /// * [(b256, b256)] - The newly created tuple.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::*;
    ///
    /// fn foo() {
    ///     let b512 = B512::zero();
    ///     let tuple: (b256, b256) = (b256, b256)::from(b512);
    /// }
    /// ```
    fn from(val: B512) -> (b256, b256) {
        ((val.bits)[0], (val.bits)[1])
    }
}

/// Methods on the `B512` type.
impl B512 {
    /// Initializes a new, zeroed `B512`.
    ///
    /// # Returns
    ///
    /// * [B512] - A zero value B512.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let zero = B512::new();
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bits: [b256::zero(), b256::zero()],
        }
    }

    /// Returns the underlying bits for the B512 type.
    ///
    /// # Returns
    ///
    /// * [[b256; 2]] - The two `b256`s that make up the `B512`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let zero = B512::new();
    ///     assert(zero.bits() == [b256::zero(), b256::zero()]);
    /// }
    /// ```
    pub fn bits(self) -> [b256; 2] {
        self.bits
    }

    /// Returns the zero value for the `B512` type.
    ///
    /// # Returns
    ///
    /// * [B512] -> The zero value for the `B512` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let zero_b512 = B512::zero();
    ///     assert(zero_b512 == B512::from((b256::zero(), b256::zero())));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bits: [b256::zero(), b256::zero()],
        }
    }

    /// Returns whether a `B512` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `B512` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let zero_b512 = B512::zero();
    ///     assert(zero_b512.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        (self.bits)[0] == b256::zero() && (self.bits)[1] == b256::zero()
    }
}

impl TryFrom<Bytes> for B512 {
    /// Casts raw `Bytes` data to an `B512`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The raw `Bytes` data to be casted.
    ///
    /// # Returns
    ///
    /// * [B512] - The newly created `B512` from the raw `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo(bytes: Bytes) {
    ///    let result = B512::try_from(bytes);
    ///    assert(result.is_some());
    ///    let b512 = result.unwrap();
    /// }
    /// ```
    fn try_from(bytes: Bytes) -> Option<Self> {
        if bytes.len() != 64 {
            return None;
        }

        Some(Self {
            bits: asm(ptr: bytes.ptr()) {
                ptr: [b256; 2]
            },
        })
    }
}

impl Into<Bytes> for B512 {
    /// Casts an `B512` to raw `Bytes` data.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The underlying raw `Bytes` data of the `B512`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let b512 = B512::from((b256::zero(), b256::zero()));
    ///     let bytes_data: Bytes = b512.into()
    ///     assert(bytes_data.len() == 64);
    /// }
    /// ```
    fn into(self) -> Bytes {
        Bytes::from(raw_slice::from_parts::<u8>(__addr_of(self.bits), 64))
    }
}

impl Hash for B512 {
    fn hash(self, ref mut state: Hasher) {
        // We want to hash just the raw bytes of the b512,
        // and not the `self.bits` array itself.
        // The fact that the signature bits are stored in a fixed-size array
        // is just an implementation detail.
        // The hash is computed only over the raw bytes of the b512.
        state.write_raw_slice(raw_slice::from_parts::<u8>(__addr_of(self.bits), 64));
    }
}
