library i8;

use core::num::*;
use ::assert::assert;

/// The 8-bit signed integer type.
/// Represented as an underlying u8 value.
/// Max value is 2 ^ 7 - 1, min value is - 2 ^ 7
pub struct I8 {
    underlying: u8,
}

pub trait From {
    /// Function for creating I8 from u8
    fn from(value: u8) -> Self;
}

impl From for I8 {
    fn from(value: u8) -> I8 {
        I8 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for I8 {
    fn eq(self, other: I8) -> bool {
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
    pub fn indent() -> u8 {
        128u8
    }
}

impl I8 {
    /// Initializes a new, zeroed I8.
    pub fn new() -> I8 {
        I8 {
            underlying: ~I8::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> I8 {
        I8 {
            underlying: ~u8::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> I8 {
        I8 {
            underlying: ~u8::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        8
    }

    pub fn neg_from(value: u8) -> I8 {
        I8 {
            underlying: ~I8::indent() - value,
        }
    }

    fn from_uint(value: u8) -> I8 {
        let underlying: u8 = value + ~I8::indent(); // as the minimal value of I8 is -~I8::indent() (1 << 7) we should add ~I8::indent() (1 << 7) 
        I8 {
            underlying
        }
    }
}

impl core::ops::Add for I8 {
    /// Add a I8 to a I8. Panics on overflow.
    fn add(self, other: Self) -> Self {
        ~I8::from(self.underlying - ~I8::indent() + other.underlying) // subtract 1 << 7 to avoid double move
    }
}

impl core::ops::Subtract for I8 {
    /// Subtract a I8 from a I8. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        let mut res = ~I8::new();
        if self > other {
            res = ~I8::from(self.underlying - other.underlying + ~I8::indent()); // add 1 << 7 to avoid loosing the move
        } else {
            res = ~I8::from(~I8::indent() - (other.underlying - self.underlying)); // subtract from 1 << 7 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for I8 {
    /// Multiply a I8 with a I8. Panics of overflow.
    fn multiply(self, other: Self) -> Self {
        let mut res = ~I8::new();
        if self.underlying >= ~I8::indent() && other.underlying >= ~I8::indent() {
            res = ~I8::from((self.underlying - ~I8::indent()) * (other.underlying -~I8::indent()) + ~I8::indent());
        } else if self.underlying < ~I8::indent() && other.underlying < ~I8::indent() {
            res = ~I8::from((~I8::indent() - self.underlying) * (~I8::indent() - other.underlying) + ~I8::indent());
        } else if self.underlying >= ~I8::indent() && other.underlying < ~I8::indent() {
            res = ~I8::from(~I8::indent() - (self.underlying - ~I8::indent()) * (~I8::indent() - other.underlying));
        } else if self.underlying < ~I8::indent() && other.underlying >= ~I8::indent() {
            res = ~I8::from(~I8::indent() - (other.underlying - ~I8::indent()) * (~I8::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for I8 {
    /// Divide a I8 by a I8. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~I8::new());
        let mut res = ~I8::new();
        if self.underlying >= ~I8::indent() && divisor.underlying > ~I8::indent() {
            res = ~I8::from((self.underlying - ~I8::indent()) / (divisor.underlying -~I8::indent()) + ~I8::indent());
        } else if self.underlying < ~I8::indent() && divisor.underlying < ~I8::indent() {
            res = ~I8::from((~I8::indent() - self.underlying) / (~I8::indent() - divisor.underlying) + ~I8::indent());
        } else if self.underlying >= ~I8::indent() && divisor.underlying < ~I8::indent() {
            res = ~I8::from(~I8::indent() - (self.underlying - ~I8::indent()) / (~I8::indent() - divisor.underlying));
        } else if self.underlying < ~I8::indent() && divisor.underlying > ~I8::indent() {
            res = ~I8::from(~I8::indent() - (~I8::indent() - self.underlying) / (divisor.underlying - ~I8::indent()));
        }
        res
    }
}
