library i16;

use core::num::*;
use ::assert::assert;

/// The 16-bit signed integer type.
/// Represented as an underlying u16 value.
pub struct I16 {
    underlying: u16,
}

pub trait From {
    /// Function for creating I16 from u16
    fn from(value: u16) -> Self;
}

impl From for I16 {
    fn from(value: u16) -> I16 {
        I16 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for I16 {
    pub fn eq(self, other: I16) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for I16 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl I16 {
    pub fn indent() -> u16 {
        32768u16
    }
}

impl I16 {
    /// Initializes a new, zeroed I16.
    pub fn new() -> I16 {
        I16 {
            underlying: ~I16::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> I16 {
        I16 {
            underlying: ~u16::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> I16 {
        I16 {
            underlying: ~u16::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        16
    }

    pub fn neg_from(value: u16) -> I16 {
        I16 {
            underlying: ~I16::indent() - value,
        }
    }

    fn from_uint(value: u16) -> I16 {
        let underlying: u16 = value + ~I16::indent(); // as the minimal value of I16 is -~I16::indent() (1 << 15) we should add ~I16::indent() (1 << 15) 
        I16 {
            underlying
        }
    }
}

impl core::ops::Add for I16 {
    /// Add a I16 to a I16. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        ~I16::from(self.underlying - ~I16::indent() + other.underlying) // subtract 1 << 15 to avoid double move
    }
}

impl core::ops::Subtract for I16 {
    /// Subtract a I16 from a I16. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~I16::new();
        if self > other {
            res = ~I16::from(self.underlying - other.underlying + ~I16::indent()); // add 1 << 15 to avoid loosing the move
        } else {
            res = ~I16::from(~I16::indent() - (other.underlying - self.underlying)); // subtract from 1 << 15 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for I16 {
    /// Multiply a I16 with a I16. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~I16::new();
        if self.underlying >= ~I16::indent() && other.underlying >= ~I16::indent() {
            res = ~I16::from((self.underlying - ~I16::indent()) * (other.underlying -~I16::indent()) + ~I16::indent());
        } else if self.underlying < ~I16::indent() && other.underlying < ~I16::indent() {
            res = ~I16::from((~I16::indent() - self.underlying) * (~I16::indent() - other.underlying) + ~I16::indent());
        } else if self.underlying >= ~I16::indent() && other.underlying < ~I16::indent() {
            res = ~I16::from(~I16::indent() - (self.underlying - ~I16::indent()) * (~I16::indent() - other.underlying));
        } else if self.underlying < ~I16::indent() && other.underlying >= ~I16::indent() {
            res = ~I16::from(~I16::indent() - (other.underlying - ~I16::indent()) * (~I16::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for I16 {
    /// Divide a I16 by a I16. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~I16::new());
        let mut res = ~I16::new();
        if self.underlying >= ~I16::indent() && divisor.underlying > ~I16::indent() {
            res = ~I16::from((self.underlying - ~I16::indent()) / (divisor.underlying -~I16::indent()) + ~I16::indent());
        } else if self.underlying < ~I16::indent() && divisor.underlying < ~I16::indent() {
            res = ~I16::from((~I16::indent() - self.underlying) / (~I16::indent() - divisor.underlying) + ~I16::indent());
        } else if self.underlying >= ~I16::indent() && divisor.underlying < ~I16::indent() {
            res = ~I16::from(~I16::indent() - (self.underlying - ~I16::indent()) / (~I16::indent() - divisor.underlying));
        } else if self.underlying < ~I16::indent() && divisor.underlying > ~I16::indent() {
            res = ~I16::from(~I16::indent() - (~I16::indent() - self.underlying) / (divisor.underlying - ~I16::indent()));
        }
        res
    }
}
