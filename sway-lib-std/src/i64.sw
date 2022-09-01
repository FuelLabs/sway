library i64;

use core::num::*;
use ::assert::assert;

/// The 64-bit signed integer type.
/// Represented as an underlying u64 value.
/// Actual value is underlying value minus 2 ^ 63
/// Max value is 2 ^ 63 - 1, min value is - 2 ^ 63
pub struct I64 {
    underlying: u64,
}

pub trait From {
    /// Function for creating I64 from u64
    fn from(underlying: u64) -> Self;
}

impl From for I64 {
    /// Helper function to get a signed number from with an underlying
    fn from(underlying: u64) -> Self {
        I64 {
            underlying,
        }
    }
}

impl core::ops::Eq for I64 {
    fn eq(self, other: I64) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for I64 {
    fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl I64 {
    /// The underlying value that corresponds to zero signed value
    pub fn indent() -> u64 {
        9223372036854775808u64
    }
}

impl I64 {
    /// Initializes a new, zeroed I64.
    pub fn new() -> I64 {
        I64 {
            underlying: ~I64::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> I64 {
        I64 {
            underlying: ~u64::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> I64 {
        I64 {
            underlying: ~u64::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        64
    }

    /// Helper function to get a negative value of unsigned number
    pub fn neg_from(value: u64) -> I64 {
        I64 {
            underlying: ~I64::indent() - value,
        }
    }

    /// Helper function to get a positive value from unsigned number
    fn from_uint(value: u64) -> I64 {
        // as the minimal value of I64 is -~I64::indent() (1 << 63) we should add ~I64::indent() (1 << 63) 
        let underlying: u64 = value + ~I64::indent();
        I64 {
            underlying
        }
    }
}

impl core::ops::Add for I64 {
    /// Add a I64 to a I64. Panics on overflow.
    fn add(self, other: Self) -> Self {
        // subtract 1 << 63 to avoid double move
        ~I64::from(self.underlying - ~I64::indent() + other.underlying)
    }
}

impl core::ops::Subtract for I64 {
    /// Subtract a I64 from a I64. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        let mut res = ~I64::new();
        if self > other {
            // add 1 << 63 to avoid loosing the move
            res = ~I64::from(self.underlying - other.underlying + ~I64::indent());
        } else {
            // subtract from 1 << 63 as we are getting a negative value
            res = ~I64::from(~I64::indent() - (other.underlying - self.underlying));
        }
        res
    }
}

impl core::ops::Multiply for I64 {
    /// Multiply a I64 with a I64. Panics of overflow.
    fn multiply(self, other: Self) -> Self {
        let mut res = ~I64::new();
        if self.underlying >= ~I64::indent() && other.underlying >= ~I64::indent() {
            res = ~I64::from((self.underlying - ~I64::indent()) * (other.underlying -~I64::indent()) + ~I64::indent());
        } else if self.underlying < ~I64::indent() && other.underlying < ~I64::indent() {
            res = ~I64::from((~I64::indent() - self.underlying) * (~I64::indent() - other.underlying) + ~I64::indent());
        } else if self.underlying >= ~I64::indent() && other.underlying < ~I64::indent() {
            res = ~I64::from(~I64::indent() - (self.underlying - ~I64::indent()) * (~I64::indent() - other.underlying));
        } else if self.underlying < ~I64::indent() && other.underlying >= ~I64::indent() {
            res = ~I64::from(~I64::indent() - (other.underlying - ~I64::indent()) * (~I64::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for I64 {
    /// Divide a I64 by a I64. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~I64::new());
        let mut res = ~I64::new();
        if self.underlying >= ~I64::indent() && divisor.underlying > ~I64::indent() {
            res = ~I64::from((self.underlying - ~I64::indent()) / (divisor.underlying -~I64::indent()) + ~I64::indent());
        } else if self.underlying < ~I64::indent() && divisor.underlying < ~I64::indent() {
            res = ~I64::from((~I64::indent() - self.underlying) / (~I64::indent() - divisor.underlying) + ~I64::indent());
        } else if self.underlying >= ~I64::indent() && divisor.underlying < ~I64::indent() {
            res = ~I64::from(~I64::indent() - (self.underlying - ~I64::indent()) / (~I64::indent() - divisor.underlying));
        } else if self.underlying < ~I64::indent() && divisor.underlying > ~I64::indent() {
            res = ~I64::from(~I64::indent() - (~I64::indent() - self.underlying) / (divisor.underlying - ~I64::indent()));
        }
        res
    }
}
