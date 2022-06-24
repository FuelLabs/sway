library u256;

use core::num::*;
use ::assert::assert;
use ::flags::*;

/// The 256-bit unsigned integer type.
/// Represented as four u64-bit components: `(a, b, c, d)`, where `value = (a << 192) + (b << 128) + (c << 64) + d`.
pub struct U256 {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

pub trait From {
    /// Function for creating U256 from its u64 components.
    pub fn from(a: u64, b: u64, c: u64, d: u64) -> Self;
} {
    fn into(val: U256) -> (u64, u64, u64, u64) {
        (val.a, val.b, val.c, val.d)
    }
}

impl From for U256 {
    pub fn from(a: u64, b: u64, c: u64, d: u64) -> U256 {
        U256 {
            a,
            b,
            c,
            d,
        }
    }
}
