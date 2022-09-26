library i32;

use core::num::*;
use ::assert::assert;

/// The 32-bit signed integer type.
/// Represented as an underlying u32 value.
/// Actual value is underlying value minus 2 ^ 31
/// Max value is 2 ^ 31 - 1, min value is - 2 ^ 31
pub struct I32 {
    underlying: u32,
}

pub trait From {
    /// Function for creating I32 from u32
    fn from(underlying: u32) -> Self;
}

impl From for I32 {
    /// Helper function to get a signed number from with an underlying
    fn from(underlying: u32) -> Self {
        Self {
            underlying,
        }
    }
}

impl core::ops::Eq for I32 {
    fn eq(self, other: Self) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for I32 {
    fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl I32 {
    /// The underlying value that corresponds to zero signed value
    pub fn indent() -> u32 {
        2147483648u32
    }
}

impl I32 {
    /// Initializes a new, zeroed I32.
    pub fn new() -> Self {
        Self {
            underlying: ~Self::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> Self {
        Self {
            underlying: ~u32::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> Self {
        Self {
            underlying: ~u32::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        32
    }

    /// Helper function to get a negative value of unsigned numbers
    pub fn neg_from(value: u32) -> Self {
        Self {
            underlying: ~Self::indent() - value,
        }
    }

    /// Helper function to get a positive value from unsigned number
    fn from_uint(value: u32) -> Self {
        // as the minimal value of I32 is 2147483648 (1 << 31) we should add ~I32::indent() (1 << 31) 
        let underlying: u32 = value + ~Self::indent();
        Self {
            underlying
        }
    }
}

impl core::ops::Add for I32 {
    /// Add a I32 to a I32. Panics on overflow.
    fn add(self, other: Self) -> Self {
        // subtract 1 << 31 to avoid double move
        ~Self::from(self.underlying - ~Self::indent() + other.underlying)
    }
}

impl core::ops::Subtract for I32 {
    /// Subtract a I32 from a I32. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        let mut res = ~Self::new();
        if self > other {
            // add 1 << 31 to avoid loosing the move
            res = ~Self::from(self.underlying - other.underlying + ~Self::indent());
        } else {
            // subtract from 1 << 31 as we are getting a negative value
            res = ~Self::from(~Self::indent() - (other.underlying - self.underlying));
        }
        res
    }
}

impl core::ops::Multiply for I32 {
    /// Multiply a I32 with a I32. Panics of overflow.
    fn multiply(self, other: Self) -> Self {
        let mut res = ~Self::new();
        if self.underlying >= ~Self::indent() && other.underlying >= ~Self::indent() {
            res = ~Self::from((self.underlying - ~Self::indent()) * (other.underlying -~Self::indent()) + ~Self::indent());
        } else if self.underlying < ~Self::indent() && other.underlying < ~Self::indent() {
            res = ~Self::from((~Self::indent() - self.underlying) * (~Self::indent() - other.underlying) + ~Self::indent());
        } else if self.underlying >= ~Self::indent() && other.underlying < ~Self::indent() {
            res = ~Self::from(~Self::indent() - (self.underlying - ~Self::indent()) * (~Self::indent() - other.underlying));
        } else if self.underlying < ~Self::indent() && other.underlying >= ~Self::indent() {
            res = ~Self::from(~Self::indent() - (other.underlying - ~Self::indent()) * (~Self::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for I32 {
    /// Divide a I32 by a I32. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~Self::new());
        let mut res = ~Self::new();
        if self.underlying >= ~Self::indent() && divisor.underlying > ~Self::indent() {
            res = ~Self::from((self.underlying - ~Self::indent()) / (divisor.underlying -~Self::indent()) + ~Self::indent());
        } else if self.underlying < ~Self::indent() && divisor.underlying < ~Self::indent() {
            res = ~Self::from((~Self::indent() - self.underlying) / (~Self::indent() - divisor.underlying) + ~Self::indent());
        } else if self.underlying >= ~Self::indent() && divisor.underlying < ~Self::indent() {
            res = ~Self::from(~Self::indent() - (self.underlying - ~Self::indent()) / (~Self::indent() - divisor.underlying));
        } else if self.underlying < ~Self::indent() && divisor.underlying > ~Self::indent() {
            res = ~Self::from(~Self::indent() - (~Self::indent() - self.underlying) / (divisor.underlying - ~Self::indent()));
        }
        res
    }
}
