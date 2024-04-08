library;

use ::convert::{TryFrom, TryInto, *};
use ::option::Option::{self, *};

impl u64 {
    /// Attempts to convert the u64 value into a u8 value.
    ///
    /// # Additional Information
    ///
    /// The max value a u8 can represent is 255.
    ///
    /// # Returns
    ///
    /// [Option<u8>] - `Some(u8)` if the u64 is less than or equal to the max u8 value. Else `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 255_u64.try_as_u8();
    ///     assert(val == Some(255_u8));
    ///
    ///     // Conversion fails as value is above the max a u8 can represent.
    ///     let val2 = 256_u64.try_as_u8();
    ///     assert(val == None);
    /// }
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u64() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }

    /// Attempts to convert the u64 value into a u16 value.
    ///
    /// # Additional Information
    ///
    /// The max value a u16 can represent is 65_535.
    ///
    /// # Returns
    ///
    /// [Option<u16>] - `Some(u16)` if the u64 is less than or equal to the max u16 value. Else `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 65_535_u64.try_as_u16();
    ///     assert(val == Some(65_535_u16));
    ///
    ///     // Conversion fails as value is above the max a u16 can represent.
    ///     let val2 = 65_536_u64.try_as_u16();
    ///     assert(val == None);
    /// }
    pub fn try_as_u16(self) -> Option<u16> {
        if self <= u16::max().as_u64() {
            Some(asm(input: self) {
                input: u16
            })
        } else {
            None
        }
    }

    /// Attempts to convert the u64 value into a u32 value.
    ///
    /// # Additional Information
    ///
    /// The max value a u32 can represent is 4_294_967_295.
    ///
    /// # Returns
    ///
    /// [Option<u32>] - `Some(u32)` if the u64 is less than or equal to the max u32 value. Else `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 4_294_967_295_u64.try_as_u32();
    ///     assert(val == Some(4_294_967_295_u32));
    ///
    ///     // Conversion fails as value is above the max a u32 can represent.
    ///     let val2 = 4_294_967_296_u64.try_as_u32();
    ///     assert(val == None);
    /// }
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
