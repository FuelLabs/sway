library;

use ::convert::TryFrom;
use ::option::Option::{self, *};
use ::ops::*;

impl u8 {
    /// Extends a `u8` to a `u16`.
    ///
    /// # Returns
    ///
    /// * [u16] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u16();
    ///     assert(result == 2u16);
    /// }
    /// ```
    pub fn as_u16(self) -> u16 {
        asm(input: self) {
            input: u16
        }
    }

    /// Extends a `u8` to a `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u32();
    ///     assert(result == 2u32);
    /// }
    /// ```
    pub fn as_u32(self) -> u32 {
        asm(input: self) {
            input: u32
        }
    }

    /// Extends a `u8` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u64();
    ///     assert(result == 2);
    /// }
    /// ```
    pub fn as_u64(self) -> u64 {
        asm(input: self) {
            input: u64
        }
    }

    /// Extends a `u8` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        let input = (0u64, 0u64, 0u64, self.as_u64());
        asm(input: input) {
            input: u256
        }
    }
}

impl TryFrom<u16> for u8 {
    fn try_from(u: u16) -> Option<Self> {
        if u > u8::max().as_u16() {
            None
        } else {
            Some(asm(r1: u) {
                r1: u8
            })
        }
    }
}

impl TryFrom<u32> for u8 {
    fn try_from(u: u32) -> Option<Self> {
        if u > u8::max().as_u32() {
            None
        } else {
            Some(asm(r1: u) {
                r1: u8
            })
        }
    }
}

impl TryFrom<u64> for u8 {
    fn try_from(u: u64) -> Option<Self> {
        if u > u8::max().as_u64() {
            None
        } else {
            Some(asm(r1: u) {
                r1: u8
            })
        }
    }
}

impl TryFrom<u256> for u8 {
    fn try_from(u: u256) -> Option<Self> {
        let parts = asm(r1: u) {
            r1: (u64, u64, u64, u64)
        };

        if parts.0 != 0
            || parts.1 != 0
            || parts.2 != 0
            || parts.3 > u8::max().as_u64()
        {
            None
        } else {
            Some(asm(r1: parts.3) {
                r1: u8
            })
        }
    }
}
