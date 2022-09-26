library i8;

use core::num::*;
use ::assert::assert;

/// The 8-bit signed integer type.
/// Represented as an underlying u8 value.
/// Actual value is underlying value minus 2 ^ 7
/// Max value is 2 ^ 7 - 1, min value is - 2 ^ 7
pub struct I8 {
    underlying: u8,
}

pub trait From {
    /// Function for creating I8 from u8
    fn from(underlying: u8) -> Self;
}

impl From for I8 {
    /// Helper function to get a signed number from with an underlying
    fn from(underlying: u8) -> Self {
        Self {
            underlying
        }
    }
}

impl core::ops::Eq for I8 {
    fn eq(self, other: Self) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for I8 {
    fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl I8 {
    /// The underlying value that corresponds to zero signed value
    pub fn indent() -> u8 {
        128u8
    }
}

impl I8 {
    /// Initializes a new, zeroed I8.
    pub fn new() -> Self {
        Self {
            underlying: ~Self::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> Self {
        Self {
            underlying: ~u8::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> Self {
        Self {
            underlying: ~u8::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        8
    }

    /// Helper function to get a negative value of unsigned number
    pub fn neg_from(value: u8) -> Self {
        Self {
            underlying: ~Self::indent() - value,
        }
    }

    /// Helper function to get a positive value from unsigned number
    fn from_uint(value: u8) -> Self {
        let underlying: u8 = value + ~Self::indent(); // as the minimal value of I8 is -~I8::indent() (1 << 7) we should add ~I8::indent() (1 << 7) 
        Self {
            underlying
        }
    }
}

impl core::ops::Add for I8 {
    /// Add a I8 to a I8. Panics on overflow.
    fn add(self, other: Self) -> Self {
        ~Self::from(self.underlying - ~Self::indent() + other.underlying) // subtract 1 << 7 to avoid double move
    }
}

impl core::ops::Subtract for I8 {
    /// Subtract a I8 from a I8. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        let mut res = ~Self::new();
        if self > other {
            res = ~Self::from(self.underlying - other.underlying + ~Self::indent()); // add 1 << 7 to avoid loosing the move
        } else {
            res = ~Self::from(~Self::indent() - (other.underlying - self.underlying)); // subtract from 1 << 7 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for I8 {
    /// Multiply a I8 with a I8. Panics of overflow.
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

impl core::ops::Divide for I8 {
    /// Divide a I8 by a I8. Panics if divisor is zero.
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
