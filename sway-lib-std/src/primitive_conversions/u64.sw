library;

use ::convert::{TryFrom, TryInto, *};
use ::option::Option::{self, *};
use ::u128::U128;

impl u64 {
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u64() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }

    pub fn try_as_u16(self) -> Option<u16> {
        if self <= u16::max().as_u64() {
            Some(asm(input: self) {
                input: u16
            })
        } else {
            None
        }
    }

    pub fn try_as_u32(self) -> Option<u32> {
        if self <= u32::max().as_u64() {
            Some(asm(input: self) {
                input: u32
            })
        } else {
            None
        }
    }
}

impl From<u8> for u64 {
    /// Casts a `u8` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The `u64` representation of the `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let u64_value = u64::from(0u8);
    /// }
    /// ```
    fn from(u: u8) -> Self {
        asm(r1: u) {
            r1: u64
        }
    }
}

impl From<u16> for u64 {
    /// Casts a `u16` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The `u64` representation of the `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let u64_value = u64::from(0u16);
    /// }
    /// ```
    fn from(u: u16) -> Self {
        asm(r1: u) {
            r1: u64
        }
    }
}

impl From<u32> for u64 {
    /// Casts a `u32` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The `u64` representation of the `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let u64_value = u64::from(0u32);
    /// }
    /// ```
    fn from(u: u32) -> Self {
        asm(r1: u) {
            r1: u64
        }
    }
}

impl TryFrom<u256> for u64 {
    fn try_from(u: u256) -> Option<Self> {
        let parts = asm(r1: u) {
            r1: (u64, u64, u64, u64)
        };

        if parts.0 != 0 || parts.1 != 0 || parts.2 != 0 {
            None
        } else {
            Some(parts.3)
        }
    }
}

impl TryFrom<U128> for u64 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            Some(u.lower())
        } else {
            None
        }
    }
}

// TODO: Replace <u64 as From<T>> with u64::from when https://github.com/FuelLabs/sway/issues/5798 is resolved.
#[test]
fn test_u64_from_u8() {
    use ::assert::assert;

    let u8_1: u8 = 0u8;
    let u8_2: u8 = 255u8;

    let u64_1 = <u64 as From<u8>>::from(u8_1);
    let u64_2 = <u64 as From<u8>>::from(u8_2);

    assert(u64_1 == 0u64);
    assert(u64_2 == 255u64);
}

#[test]
fn test_u64_from_u16() {
    use ::assert::assert;

    let u16_1: u16 = 0u16;
    let u16_2: u16 = 65535u16;

    let u64_1 = <u64 as From<u16>>::from(u16_1);
    let u64_2 = <u64 as From<u16>>::from(u16_2);

    assert(u64_1 == 0u64);
    assert(u64_2 == 65535u64);
}

#[test]
fn test_u64_from_u32() {
    use ::assert::assert;

    let u32_1: u32 = 0u32;
    let u32_2: u32 = 4294967295u32;

    let u64_1 = <u64 as From<u32>>::from(u32_1);
    let u64_2 = <u64 as From<u32>>::from(u32_2);

    assert(u64_1 == 0u64);
    assert(u64_2 == 4294967295u64);
}

#[test]
fn test_u64_try_from_u256() {
    use ::assert::assert;

    let u256_1 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_2 = 0x1000000000000000000000000000000000000000000000000000000000000000u256;

    let u64_1 = u64::try_from(u256_1);
    let u64_2 = u64::try_from(u256_2);

    assert(u64_1.is_some());
    assert(u64_1.unwrap() == 2);

    assert(u64_2.is_none());
}

#[test]
fn test_u64_try_from_u128() {
    use ::assert::assert;

    let u128_1: U128 = U128::new();
    let u128_2: U128 = U128::from((1, 0));

    let u64_1 = <u64 as TryFrom<U128>>::try_from(u128_1);
    let u64_2 = <u64 as TryFrom<U128>>::try_from(u128_2);

    assert(u64_1.is_some());
    assert(u64_1.unwrap() == 0u64);

    assert(u64_2.is_none());
}
