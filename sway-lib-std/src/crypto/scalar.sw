library;

use ::convert::{From, TryFrom};
use ::bytes::{Bytes, *};
use ::option::Option::{self, *};

// NOTE: Bytes are used to support numbers greater than 32 bytes for future curves.
/// The Scalar type used in cryptographic operations.
pub struct Scalar {
    bytes: Bytes,
}

impl Eq for Scalar {
    fn eq(self, other: Self) -> bool {
        // All scalars must be of length 32
        if self.bytes.len() != 32 || other.bytes.len() != 32 {
            return false;
        }

        let mut iter = 0;
        while iter < 32 {
            if self.bytes.get(iter).unwrap() != other.bytes.get(iter).unwrap()
            {
                return false;
            }

            iter += 1;
        }
        true
    }
}

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
        self == Self::zero()
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
