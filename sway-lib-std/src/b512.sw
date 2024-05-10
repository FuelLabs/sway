//! A wrapper around two `b256` types to support the usage of 64-byte values in Sway,
//! which are needed when working with public keys and signatures.
library;

use ::convert::From;
use ::zero::Zero;

/// Stores two `b256`s in contiguous memory.
/// Guaranteed to be contiguous for use with ec-recover: `std::ecr::ec_recover`.
pub struct B512 {
    /// The two `b256`s that make up the `B512`.
    bits: [b256; 2],
}

impl core::ops::Eq for B512 {
    fn eq(self, other: Self) -> bool {
        (self.bits)[0] == (other.bits)[0] && (self.bits)[1] == (other.bits)[1]
    }
}

/// Functions for casting between `B512` and an array of two `b256`s.
impl From<(b256, b256)> for B512 {
    fn from(components: (b256, b256)) -> Self {
        Self {
            bits: [components.0, components.1],
        }
    }
}

impl From<B512> for (b256, b256) {
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
}

impl Zero for B512 {
    fn zero() -> Self {
        Self {
            bits: [b256::zero(), b256::zero()],
        }
    }

    fn is_zero(self) -> bool {
        (self.bits)[0] == b256::zero() && (self.bits)[1] == b256::zero()
    }
}
