library;

use ::bytes::Bytes;
use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::u128::U128;

impl TryFrom<Bytes> for b256 {
    fn try_from(b: Bytes) -> Option<Self> {
        if b.len() > 32 {
            None
        } else {
            let mut val = 0x0000000000000000000000000000000000000000000000000000000000000000;
            let ptr = __addr_of(val);
            b.ptr().copy_to::<b256>(ptr, 1);
            Some(val)
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
        num.as_b256()
    }
}

impl From<U128> for b256 {
    /// Converts a `U128` to a `b256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [U128] - The `U128` to be converted.
    ///
    /// # Returns
    ///
    /// * [b256] - The `b256` representation of the `U128` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///    let u128_value = U128::from((18446744073709551615_u64, 18446744073709551615_u64));
    ///    let b256_value = b256::from(u128_value);
    /// }
    /// ```
    fn from(num: U128) -> Self {
        let input = (0u64, 0u64, num.upper(), num.lower());
        asm(input: input) {
            input: b256
        }
    }
}

#[test]
fn test_b256_try_from_bytes() {
    use ::assert::assert;

    let mut initial_bytes = Bytes::with_capacity(32);
    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        initial_bytes.push(51u8);
        i += 1;
    }
    let res = b256::try_from(initial_bytes);
    let expected = 0x3333333333333333333333333333333333333333333333333333333333333333;

    assert(res.unwrap() == expected);

    let mut second_bytes = Bytes::with_capacity(33);
    i = 0;
    while i < 33 {
        // 0x33 is 51 in decimal
        second_bytes.push(51u8);
        i += 1;
    }
    let res = b256::try_from(second_bytes);
    assert(res.is_none());

    // bytes is still available to use:
    assert(second_bytes.len() == 33);
    assert(second_bytes.capacity() == 33);
}

#[test]
fn test_b256_from_u256() {
    use ::assert::assert;

    let val = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let res = b256::from(val);
    assert(res == 0x0000000000000000000000000000000000000000000000000000000000000000);
}

#[test]
fn test_b256_from_u128() {
    use ::assert::assert;

    let b256_value = <b256 as From<U128>>::from(U128::from((18446744073709551615_u64, 18446744073709551615_u64)));
    assert(
        b256_value == 0x00000000000000000000000000000000ffffffffffffffffffffffffffffffff,
    );
}
