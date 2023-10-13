library;

use ::assert::assert;
use ::bytes::Bytes;
use ::alloc::alloc;
use ::option::Option;
use ::bytes_conversions::u64::*;

impl u256 {
    /// Converts the `u256` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [Bytes] - The 32 bytes that compose the `u256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256;
    ///     let bytes = x.to_le_bytes();
    ///
    ///     let mut i: u8 = 0;
    ///     while i < 32_u8 {
    ///         assert(bytes.get(i.as_u64()).unwrap() == 32_u8 - i);
    ///         i += 1_u8;
    ///     }
    /// }
    /// ```
    pub fn to_le_bytes(self) -> Bytes {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {r1: (u64, u64, u64, u64)};
        let a = a.to_le_bytes();
        let b = b.to_le_bytes();
        let c = c.to_le_bytes();
        let d = d.to_le_bytes();

        let (mut a, mut b, mut c, mut d) = (d,c,b,a);

        a.append(b);
        a.append(c);
        a.append(d);

        a        
    }

    /// Converts a sequence of little-endian bytes to a `u256`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - The 32 bytes that compose the `u256`.
    ///
    /// # Returns
    /// 
    /// * [u256] - The resulting `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let mut bytes = Bytes::with_capacity(32);
    ///     let mut i: u8 = 0;
    ///     while i < 32_u8 {
    ///         bytes.push(32_u8 - i);
    ///         i += 1_u8;
    ///     }
    ///
    ///     let x = u256::from_le_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 32);

        let (a_bytes, rest) = bytes.split_at(8);
        let (b_bytes, rest) = rest.split_at(8);
        let (c_bytes, d_bytes) = rest.split_at(8);

        let a = u64::from_le_bytes(a_bytes);
        let b = u64::from_le_bytes(b_bytes);
        let c = u64::from_le_bytes(c_bytes);
        let d = u64::from_le_bytes(d_bytes);

        let result = (d, c, b, a);

        asm(r1: result) {
            r1: u256
        }
    }

    /// Converts the `u256` to a sequence of big-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [Bytes] - The 32 bytes that compose the `u256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256;
    ///     let bytes = x.to_be_bytes();
    ///
    ///     let mut i: u8 = 0;
    ///     while i < 32_u8 {
    ///         assert(bytes.get(i.as_u64()).unwrap() == i + 1_u8);
    ///         i += 1_u8;
    ///     }
    /// }
    /// ```
    pub fn to_be_bytes(self) -> Bytes {
        let b: b256 = asm(r1: self) {r1: b256};
        Bytes::from(b)
    }

    /// Converts a sequence of big-endian bytes to a `u256`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - The 32 bytes that compose the `u256`.
    ///
    /// # Returns
    /// 
    /// * [u256] - The resulting `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let mut bytes = Bytes::with_capacity(32);
    ///     let mut i: u8 = 0;
    ///     while i < 32_u8 {
    ///         bytes.push(i + 1);
    ///         i += 1_u8;
    ///     }
    ///
    ///     let x = u256::from_be_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 32);
        let b: b256 = bytes.into();
        asm(r1: b) {
            r1: u256
        }
    }
}

#[test]
fn test_u256_from_le_bytes() {
    let mut bytes = Bytes::with_capacity(32);
    let mut i: u8 = 0;
    while i < 32_u8 {
        bytes.push(32_u8 - i);
        i += 1_u8;
    }

    let x = u256::from_le_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256);
}

#[test]
fn test_u256_to_le_bytes() {
    let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256;

    let bytes = x.to_le_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes.get(i.as_u64()).unwrap() == 32_u8 - i);
        i += 1_u8;
    }
}

#[test]
fn test_u256_from_be_bytes() {
    let mut bytes = Bytes::with_capacity(32);

    let mut i: u8 = 0;
    while i < 32_u8 {
        bytes.push(i + 1_u8);
        i += 1_u8;
    }

    let x = u256::from_be_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256);
}

#[test]
fn test_u256_to_be_bytes() {
    let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256;

    let bytes = x.to_be_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes.get(i.as_u64()).unwrap() == i + 1_u8);
        i += 1_u8;
    }
}
