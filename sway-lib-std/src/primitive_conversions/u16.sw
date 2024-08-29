library;

use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::u128::U128;

impl u16 {
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u16() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }
}

impl From<u8> for u16 {
    /// Casts a `u8` to a `u16`.
    ///
    /// # Returns
    ///
    /// * [u16] - The `u16` representation of the `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    ///
    /// fn foo() {
    ///     let u16_value = u16::from(0u8);
    /// }
    /// ```
    fn from(u: u8) -> Self {
        asm(r1: u) {
            r1: u16
        }
    }
}

impl TryFrom<u32> for u16 {
    fn try_from(u: u32) -> Option<Self> {
        if u > u16::max().as_u32() {
            None
        } else {
            Some(asm(r1: u) {
                r1: u16
            })
        }
    }
}

impl TryFrom<u64> for u16 {
    fn try_from(u: u64) -> Option<Self> {
        if u > u16::max().as_u64() {
            None
        } else {
            Some(asm(r1: u) {
                r1: u16
            })
        }
    }
}

impl TryFrom<u256> for u16 {
    fn try_from(u: u256) -> Option<Self> {
        let parts = asm(r1: u) {
            r1: (u64, u64, u64, u64)
        };

        if parts.0 != 0
            || parts.1 != 0
            || parts.2 != 0
            || parts.3 > u16::max().as_u64()
        {
            None
        } else {
            Some(asm(r1: parts.3) {
                r1: u16
            })
        }
    }
}

impl TryFrom<U128> for u16 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            <u16 as TryFrom<u64>>::try_from(u.lower())
        } else {
            None
        }
    }
}
