library;

use ::bytes::Bytes;
use ::convert::{From, TryFrom};
use ::option::Option::{self, *};

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
