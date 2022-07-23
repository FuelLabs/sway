library i16;

use core::num::*;
use ::assert::assert;

/// The 16-bit signed integer type.
/// Represented as an underlying u16 value.
pub struct i16 {
    underlying: u16,
}

pub trait From {
    /// Function for creating i16 from its u16 and bool components.
    fn from(value: u16) -> Self;
}

impl From for i16 {
    fn from(value: u16) -> i16 {
        i16 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for i16 {
    pub fn eq(self, other: i16) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for i16 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl i16 {
    pub fn indent() -> u16 {
        32768u16
    }
}

impl i16 {
    /// Initializes a new, zeroed i16.
    pub fn new() -> i16 {
        i16 {
            underlying: ~i16::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i16 {
        i16 {
            underlying: ~u16::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> i16 {
        i16 {
            underlying: ~u16::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        16
    }

    pub fn neg_from(value: u16) -> i16 {
        i16 {
            underlying: ~i16::indent() - value,
        }
    }

    fn from_uint(value: u16) -> i16 {
        let underlying: u16 = value + ~i16::indent(); // as the minimal value of i16 is -~i16::indent() (1 << 15) we should add ~i16::indent() (1 << 15) 
        i16 {
            underlying
        }
    }
}

impl core::ops::Add for i16 {
    /// Add a i16 to a i16. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        ~i16::from(self.underlying - ~i16::indent() + other.underlying) // subtract 1 << 15 to avoid double move
    }
}

impl core::ops::Subtract for i16 {
    /// Subtract a i16 from a i16. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~i16::new();
        if self > other {
            res = ~i16::from(self.underlying - other.underlying + ~i16::indent()); // add 1 << 15 to avoid loosing the move
        } else {
            res = ~i16::from(~i16::indent() - (other.underlying - self.underlying)); // subtract from 1 << 15 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for i16 {
    /// Multiply a i16 with a i16. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~i16::new();
        if self.underlying >= ~i16::indent() && other.underlying >= ~i16::indent() {
            res = ~i16::from((self.underlying - ~i16::indent()) * (other.underlying -~i16::indent()) + ~i16::indent());
        } else if self.underlying < ~i16::indent() && other.underlying < ~i16::indent() {
            res = ~i16::from((~i16::indent() - self.underlying) * (~i16::indent() - other.underlying) + ~i16::indent());
        } else if self.underlying >= ~i16::indent() && other.underlying < ~i16::indent() {
            res = ~i16::from(~i16::indent() - (self.underlying - ~i16::indent()) * (~i16::indent() - other.underlying));
        } else if self.underlying < ~i16::indent() && other.underlying >= ~i16::indent() {
            res = ~i16::from(~i16::indent() - (other.underlying - ~i16::indent()) * (~i16::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for i16 {
    /// Divide a i16 by a i16. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~i16::new());
        let mut res = ~i16::new();
        if self.underlying >= ~i16::indent() && divisor.underlying > ~i16::indent() {
            res = ~i16::from((self.underlying - ~i16::indent()) / (divisor.underlying -~i16::indent()) + ~i16::indent());
        } else if self.underlying < ~i16::indent() && divisor.underlying < ~i16::indent() {
            res = ~i16::from((~i16::indent() - self.underlying) / (~i16::indent() - divisor.underlying) + ~i16::indent());
        } else if self.underlying >= ~i16::indent() && divisor.underlying < ~i16::indent() {
            res = ~i16::from(~i16::indent() - (self.underlying - ~i16::indent()) / (~i16::indent() - divisor.underlying));
        } else if self.underlying < ~i16::indent() && divisor.underlying > ~i16::indent() {
            res = ~i16::from(~i16::indent() - (~i16::indent() - self.underlying) / (divisor.underlying - ~i16::indent()));
        }
        res
    }
}
