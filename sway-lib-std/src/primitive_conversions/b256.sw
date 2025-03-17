library;

use ::bytes::Bytes;
use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::b512::B512;

impl b256 {
    /// Converts a `b256` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `b256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        asm(input: self) {
            input: u256
        }
    }
}

impl TryFrom<B512> for b256 {
    /// Attempts conversion from a `B512` to a `b256`.
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
    /// * [Option<b256>] - The `b256` representation of the `B512` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::b512::B512;
    ///
    /// fn foo() {
    ///     let b512_value = B512::new();
    ///     let b256_value = b256::try_from(b512_value).unwrap();
    /// }
    /// ```
    fn try_from(val: B512) -> Option<Self> {
        let bits = val.bits();
        if bits[0] == b256::zero() {
            Some(bits[1])
        } else {
            None
        }
    }
}

impl From<u256> for b256 {
    /// Casts a `u256` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `u256`.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let b256_value = b256::from(0x0000000000000000000000000000000000000000000000000000000000000000_u256);
    /// }
    /// ```
    fn from(num: u256) -> Self {
        asm(input: num) {
            input: b256
        }
    }
}

impl From<(u64, u64, u64, u64)> for b256 {
    /// Casts a tuple of 4 `u64` values to a `b256`.
    ///
    /// # Arguments
    ///
    /// * `nums`: (u64, u64, u64, u64) - The tuple of `u64` values to be casted.
    ///
    /// # Returns
    ///
    /// * [b256] - The `b256` representation of the tuple of `u64` values.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///    let b256_value = b256::from((1, 2, 3, 4));
    /// }
    /// ```
    fn from(nums: (u64, u64, u64, u64)) -> Self {
        asm(nums: nums) {
            nums: b256
        }
    }
}
