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
    /// [Option<u16>] - `Some(u8)` if the u64 is less than or equal to the max u16 value. Else `None`.
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
