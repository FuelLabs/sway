library;

use ::convert::{From, TryFrom};
use ::option::Option::{self, *};

impl u32 {
    /// Attempts to convert the u32 value into a u8 value.
    ///
    /// # Additional Information
    ///
    /// The max value a u8 can represent is 255.
    ///
    /// # Returns
    ///
    /// [Option<u8>] - `Some(u8)` if the u32 is less than or equal to the max u8 value. Else `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 255_u32.try_as_u8();
    ///     assert(val == Some(255_u8));
    ///
    ///     // Conversion fails as value is above the max a u8 can represent.
    ///     let val2 = 256_u32.try_as_u8();
    ///     assert(val == None);
    /// }
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u32() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }

    /// Attempts to convert the u32 value into a u16 value.
    ///
    /// # Additional Information
    ///
    /// The max value a u16 can represent is 65_535.
    ///
    /// # Returns
    ///
    /// [Option<u16>] - `Some(u16)` if the u32 is less than or equal to the max u16 value. Else `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 65_535_u32.try_as_u16();
    ///     assert(val == Some(65_535_u16));
    ///
    ///     // Conversion fails as value is above the max a u16 can represent.
    ///     let val2 = 65_536_u32.try_as_u16();
    ///     assert(val == None);
    /// }
    pub fn try_as_u16(self) -> Option<u16> {
        if self <= u16::max().as_u32() {
            Some(asm(input: self) {
                input: u16
            })
        } else {
            None
        }
    }
}

impl From<u8> for u32 {
    /// Casts a `u8` to a `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The `u32` representation of the `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let u32_value = u32::from(0u8);
    /// }
    /// ```
    fn from(u: u8) -> Self {
        asm(r1: u) {
            r1: u32
        }
    }
}

impl From<u16> for u32 {
    /// Casts a `u16` to a `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The `u32` representation of the `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let u32_value = u32::from(0u16);
    /// }
    /// ```
    fn from(u: u16) -> Self {
        asm(r1: u) {
            r1: u32
        }
    }
}

impl TryFrom<u64> for u32 {
    fn try_from(u: u64) -> Option<Self> {
        if u > u32::max().as_u64() {
            None
        } else {
            Some(asm(r1: u) {
                r1: u32
            })
        }
    }
}

impl TryFrom<u256> for u32 {
    fn try_from(u: u256) -> Option<Self> {
        let parts = asm(r1: u) {
            r1: (u64, u64, u64, u64)
        };

        if parts.0 != 0
            || parts.1 != 0
            || parts.2 != 0
            || parts.3 > u32::max().as_u64()
        {
            None
        } else {
            Some(asm(r1: parts.3) {
                r1: u32
            })
        }
    }
}

// TODO: Replace <u32 as From<T>> with u32::from when https://github.com/FuelLabs/sway/issues/5798 is resolved.
#[test]
fn test_u32_from_u8() {
    use ::assert::assert;

    let u8_1: u8 = 0u8;
    let u8_2: u8 = 255u8;

    let u32_1 = <u32 as From<u8>>::from(u8_1);
    let u32_2 = <u32 as From<u8>>::from(u8_2);

    assert(u32_1 == 0u32);
    assert(u32_2 == 255u32);
}

#[test]
fn test_u32_from_u16() {
    use ::assert::assert;

    let u16_1: u16 = 0u16;
    let u16_2: u16 = 65535u16;

    let u32_1 = <u32 as From<u16>>::from(u16_1);
    let u32_2 = <u32 as From<u16>>::from(u16_2);

    assert(u32_1 == 0u32);
    assert(u32_2 == 65535u32);
}

#[test]
fn test_u32_try_from_u64() {
    use ::assert::assert;

    let u64_1: u64 = 2;
    let u64_2: u64 = u32::max().as_u64() + 1;

    let u32_1 = <u32 as TryFrom<u64>>::try_from(u64_1);
    let u32_2 = <u32 as TryFrom<u64>>::try_from(u64_2);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 2u32);

    assert(u32_2.is_none());
}

#[test]
fn test_u32_try_from_u256() {
    use ::assert::assert;

    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_2: u256 = 0x1000000000000000000000000000000000000000000000000000000000000000u256;

    let u32_1 = <u32 as TryFrom<u256>>::try_from(u256_1);
    let u32_2 = <u32 as TryFrom<u256>>::try_from(u256_2);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 2u32);

    assert(u32_2.is_none());
}
