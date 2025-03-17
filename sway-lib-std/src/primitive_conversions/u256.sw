library;

use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::b512::B512;
use ::ops::*;
use ::primitive_conversions::{b256::*, u16::*, u32::*, u64::*, u8::*};

impl u256 {
    /// Converts a `u256` to a `b256`.
    ///
    /// # Returns
    ///
    /// * [b256] - The converted `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;
    ///     let result = val.as_b256();
    ///     assert(result == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
    /// }
    /// ```
    pub fn as_b256(self) -> b256 {
        asm(input: self) {
            input: b256
        }
    }
}

impl TryFrom<B512> for u256 {
    /// Attempts conversion from a `B512` to a `u256`.
    ///
    /// # Additional Information
    ///
    /// If the high bits of the `B512` are not zero, the conversion will fail.
    ///
    /// # Arguments
    ///
    /// * `val`: [B512] - The `B512` to be converted.
    ///
    /// # Returns
    ///
    /// * [Option<u256>] - The `u256` representation of the `B512` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let b512_value = B512::new();
    ///     let u256_value = u256::try_from(b512_value).unwrap();
    /// }
    /// ```
    fn try_from(val: B512) -> Option<Self> {
        let bits = val.bits();
        if bits[0] == b256::zero() {
            Some(bits[1].as_u256())
        } else {
            None
        }
    }
}

/// Functions for casting between `u256` and other types.
impl From<u8> for u256 {
    /// Casts a `u8` to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [u8] - The `u8` to be casted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///    let u256_value = u256::from(255_u8);
    /// }
    /// ```
    fn from(num: u8) -> Self {
        num.as_u256()
    }
}

impl From<u16> for u256 {
    /// Casts a `u16` to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [u16] - The `u16` to be casted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///    let u256_value = u256::from(65535_u16);
    /// }
    /// ```
    fn from(num: u16) -> Self {
        num.as_u256()
    }
}

impl From<u32> for u256 {
    /// Casts a `u32` to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [u32] - The `u32` to be casted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///    let u256_value = u256::from(4294967295_u32);
    /// }
    /// ```
    fn from(num: u32) -> Self {
        num.as_u256()
    }
}

impl From<u64> for u256 {
    /// Casts a `u64` to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [u64] - The `u64` to be casted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///    let u256_value = u256::from(18446744073709551615_u64);
    /// }
    /// ```
    fn from(num: u64) -> Self {
        num.as_u256()
    }
}

impl From<b256> for u256 {
    /// Casts raw `b256` data to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///    let u256_value = u256::zero();
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        bits.as_u256()
    }
}

impl From<(u64, u64, u64, u64)> for u256 {
    /// Casts a tuple of 4 `u64` values to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `nums`: (u64, u64, u64, u64) - The tuple of `u64` values to be casted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the tuple of `u64` values.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///    let u256_value = u256::from((1, 2, 3, 4));
    /// }
    /// ```
    fn from(nums: (u64, u64, u64, u64)) -> Self {
        asm(nums: nums) {
            nums: u256
        }
    }
}
