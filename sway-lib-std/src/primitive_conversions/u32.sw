library;

use ::convert::{From, TryFrom};
use ::option::Option::{self, *};
use ::u128::U128;

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

impl TryFrom<U128> for u32 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            <u32 as TryFrom<u64>>::try_from(u.lower())
        } else {
            None
        }
    }
}
