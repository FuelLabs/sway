library;

use ::convert::TryFrom;
use ::option::Option::{self, *};
use ::u128::U128;

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

impl TryFrom<U128> for u8 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            <u8 as TryFrom<u64>>::try_from(u.lower())
        } else {
            None
        }
    }
}
