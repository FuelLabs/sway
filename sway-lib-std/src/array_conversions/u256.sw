library;

use ::array_conversions::u64::*;
use ::assert::assert;

impl u256 {
    /// Converts the `u256` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 32]] - An array of 32 `u8` bytes that compose the `u256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8,
    ///             22_u8, 21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8,
    ///             12_u8, 11_u8, 10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8,
    ///             2_u8, 1_u8];
    ///
    ///     let x = u256::from_le_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> [u8; 32] {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {r1: (u64, u64, u64, u64)};
        let a = a.to_le_bytes();
        let b = b.to_le_bytes();
        let c = c.to_le_bytes();
        let d = d.to_le_bytes();

        let (a,b,c,d) = (d,c,b,a);

        let output = [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
                      b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                      c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                      d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]];

        output
    }

    /// Converts a sequence of little-endian bytes to a `u256`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 32]] - A sequence of 32 `u8` bytes that represent a `u256`.
    ///
    /// # Returns
    /// 
    /// * [u256] - The resulting `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8,
    ///             22_u8, 21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8,
    ///             12_u8, 11_u8, 10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8,
    ///             2_u8, 1_u8];
    ///
    ///     let x = u256::from_le_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;
    /// ```
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        let a = u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
        let b = u64::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]);
        let c = u64::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]);
        let d = u64::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31]]);

        let result = (d, c, b, a);

        asm(r1: result) {
            r1: u256
        }
    }

    /// Converts the `u256` to a sequence of big-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 32]] - An array of 32 `u8` bytes that compose the `u256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;
    ///     let bytes = x.to_be_bytes();
    ///
    ///     let mut i: u8 = 0;
    ///     while i < 32_u8 {
    ///         assert(bytes[i.as_u64()] == i + 1_u8);
    ///         i += 1_u8;
    ///     }
    /// }
    /// ```
    pub fn to_be_bytes(self) -> [u8; 32] {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {r1: (u64, u64, u64, u64)};
        let a = a.to_be_bytes();
        let b = b.to_be_bytes();
        let c = c.to_be_bytes();
        let d = d.to_be_bytes();

        let output = [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
                      b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                      c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                      d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]];

        output
    }

    /// Converts a sequence of big-endian bytes to a `u256`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 32]] - A sequence of 32 `u8` bytes that represent a `u256`.
    ///
    /// # Returns
    /// 
    /// * [u256] - The resulting `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8,
    ///             11_u8, 12_u8, 13_u8, 14_u8, 15_u8, 16_u8, 17_u8, 18_u8, 19_u8, 20_u8,
    ///             21_u8, 22_u8, 23_u8, 24_u8, 25_u8, 26_u8, 27_u8, 28_u8, 29_u8, 30_u8,
    ///             31_u8, 32_u8];
    ///     let x = u256::from_be_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let a = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
        let b = u64::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]);
        let c = u64::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]);
        let d = u64::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31]]);

        let result = (a, b, c, d);

        asm(r1: result) {
            r1: u256
        }
    }
}

#[test]
fn test_u256_from_le_bytes() {
    let bytes = [32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8,
                 22_u8, 21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8,
                 12_u8, 11_u8, 10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8,
                 2_u8, 1_u8];

    let x = u256::from_le_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
}

#[test]
fn test_u256_to_le_bytes() {
    let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;

    let bytes = x.to_le_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes[i.as_u64()] == 32_u8 - i);
        i += 1_u8;
    }

}

#[test]
fn test_u256_from_be_bytes() {
    let bytes = [1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8,
                 11_u8, 12_u8, 13_u8, 14_u8, 15_u8, 16_u8, 17_u8, 18_u8, 19_u8, 20_u8,
                 21_u8, 22_u8, 23_u8, 24_u8, 25_u8, 26_u8, 27_u8, 28_u8, 29_u8, 30_u8,
                 31_u8, 32_u8];

    let x = u256::from_be_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
}

#[test]
fn test_u256_to_be_bytes() {
    let x: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;

    let bytes = x.to_be_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes[i.as_u64()] == i + 1_u8);
        i += 1_u8;
    }
}
