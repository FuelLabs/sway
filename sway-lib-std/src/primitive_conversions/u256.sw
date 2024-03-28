library;

use ::convert::From;

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
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///    let u256_value = u256::from(ZERO_B256);
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        bits.as_u256()
    }
}

#[test]
fn test_u256_from_u8() {
    use ::assert::assert;

    let u256_value = u256::from::<u8>(255_u8);
    assert(u256_value == 0x00000000000000000000000000000000000000000000000000000000000000ff_u256);
}

#[test]
fn test_u256_from_u16() {
    use ::assert::assert;

    let u256_value = u256::from::<u16>(65535_u16);
    assert(u256_value == 0x000000000000000000000000000000000000000000000000000000000000ffff_u256);
}

#[test]
fn test_u256_from_u32() {
    use ::assert::assert;

    let u256_value = u256::from::<u32>(4294967295_u32);
    assert(u256_value == 0x00000000000000000000000000000000000000000000000000000000ffffffff_u256);
}

#[test]
fn test_u256_from_u64() {
    use ::assert::assert;
    
    let u256_value = u256::from::<u64>(18446744073709551615_u64);
    assert(u256_value == 0x000000000000000000000000000000000000000000000000ffffffffffffffff_u256);
}

#[test]
fn test_u256_from_b256() {
    use ::assert::assert;
    use ::constants::ZERO_B256;

    let u256_value = u256::from::<b256>(ZERO_B256);
    assert(u256_value == 0x0000000000000000000000000000000000000000000000000000000000000000_u256);

    let u256_value = u256::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    assert(u256_value == 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_u256);
}
