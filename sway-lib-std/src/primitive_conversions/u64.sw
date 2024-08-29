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
