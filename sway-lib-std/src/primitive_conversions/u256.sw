library;

use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::u128::U128;
use ::b512::B512;

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

impl From<U128> for u256 {
    /// Converts a `U128` to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [U128] - The `U128` to be converted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `U128` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///    let u128_value = U128::from((18446744073709551615_u64, 18446744073709551615_u64));
    ///    let u256_value = u256::from(u128_value);
    /// }
    /// ```
    fn from(num: U128) -> Self {
        let input = (0u64, 0u64, num.upper(), num.lower());
        asm(input: input) {
            input: u256
        }
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

// TODO: Replace <u256 as From<T>> with u256::from when https://github.com/FuelLabs/sway/issues/5798 is resolved.
#[test]
fn test_u256_from_u8() {
    use ::assert::assert;

    let u256_value = <u256 as From<u8>>::from(255_u8);
    assert(
        u256_value == 0x00000000000000000000000000000000000000000000000000000000000000ff_u256,
    );
}

#[test]
fn test_u256_from_u16() {
    use ::assert::assert;

    let u256_value = <u256 as From<u16>>::from(65535_u16);
    assert(
        u256_value == 0x000000000000000000000000000000000000000000000000000000000000ffff_u256,
    );
}

#[test]
fn test_u256_from_u32() {
    use ::assert::assert;

    let u256_value = <u256 as From<u32>>::from(4294967295_u32);
    assert(
        u256_value == 0x00000000000000000000000000000000000000000000000000000000ffffffff_u256,
    );
}

#[test]
fn test_u256_from_u64() {
    use ::assert::assert;

    let u256_value = <u256 as From<u64>>::from(18446744073709551615_u64);
    assert(
        u256_value == 0x000000000000000000000000000000000000000000000000ffffffffffffffff_u256,
    );
}

#[test]
fn test_u256_from_b256() {
    use ::assert::assert;

    let u256_value = <u256 as From<b256>>::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(
        u256_value == 0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );

    let u256_value = <u256 as From<b256>>::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    assert(
        u256_value == 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_u256,
    );
}

#[test]
fn test_u256_from_u128() {
    use ::assert::assert;

    let u256_value = <u256 as From<U128>>::from(U128::from((18446744073709551615_u64, 18446744073709551615_u64)));
    assert(
        u256_value == 0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff_u256,
    );
}

#[test]
fn test_u256_from_tuple() {
    use ::assert::assert;

    let u256_value = <u256 as From<(u64, u64, u64, u64)>>::from((1, 2, 3, 4));
    assert(
        u256_value == 0x0000000000000001000000000000000200000000000000030000000000000004_u256,
    );
}

#[test]
fn test_u256_try_from_b512() {
    use ::assert::assert;

    let b512_value = B512::new();
    let u256_value = u256::try_from(b512_value);
    assert(u256_value.is_some());

    let b512_value = B512::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        b256::zero(),
    ));
    let u256_value = u256::try_from(b512_value);
    assert(u256_value.is_none());
}
