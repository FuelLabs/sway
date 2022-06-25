library u256;

use core::num::*;
use ::result::Result;

/// The 256-bit unsigned integer type.
/// Represented as four u64-bit components: `(a, b, c, d)`, where `value = (a << 192) + (b << 128) + (c << 64) + d`.
pub struct U256 {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

pub trait From {
    /// Function for creating a U256 from its u64 components.
    pub fn from(a: u64, b: u64, c: u64, d: u64) -> Self;
} {
    /// Function for extracting 4 u64s from a U256.
    fn into(val: U256) -> (u64, u64, u64, u64) {
        (val.a, val.b, val.c, val.d)
    }
}

impl From for U256 {
    pub fn from(a: u64, b: u64, c: u64, d: u64) -> U256 {
        U256 {
            a, b, c, d,
        }
    }
}

impl core::ops::Eq for U256 {
    /// Function for comparing 2 U256s for equality
    pub fn eq(self, other: Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
    }
}

impl U256 {
    /// Initializes a new, zeroed U256.
    pub fn new() -> U256 {
        U256 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// Downcast to `u64`. Err if precision would be lost, Ok otherwise.
    pub fn to_u64(self) -> Result<u64, ()> {
        if self.a == 0 && self.b == 0 && self.c == 0 {
            Result::Ok(self.d)
        } else {
            Result::Err(())
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> U256 {
        U256 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>256</sup> - 1.
    pub fn max() -> U256 {
        U256 {
            a: ~u64::max(),
            b: ~u64::max(),
            c: ~u64::max(),
            d: ~u64::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        256
    }
}
