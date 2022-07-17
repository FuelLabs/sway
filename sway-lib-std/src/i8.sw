library i8;

use core::num::*;
use ::assert::assert;

/// The 8-bit signed integer type.
/// Represented as an underlying u8 value.
pub struct i8 {
    underlying: u8,
}

pub trait From {
    /// Function for creating i8 from its u8 and bool components.
    fn from(value: u8) -> Self;
}

impl From for i8 {
    fn from(value: u8) -> i8 {
        i8 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for i8 {
    pub fn eq(self, other: i8) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for i8 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl i8 {
    pub fn indent() -> u8 {
        128u8
    }
}

impl i8 {
    /// Initializes a new, zeroed i8.
    pub fn new() -> i8 {
        i8 {
            underlying: ~i8::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i8 {
        i8 {
            underlying: ~u8::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> i8 {
        i8 {
            underlying: ~u8::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        8
    }

    pub fn neg_from(value: u8) -> i8 {
        i8 {
            underlying: ~i8::indent() - value,
        }
    }

    fn from_uint(value: u8) -> i8 {
        let underlying: u8 = value + ~i8::indent(); // as the minimal value of i8 is -~i8::indent() (1 << 7) we should add ~i8::indent() (1 << 7) 
        i8 {
            underlying
        }
    }
}

impl core::ops::Add for i8 {
    /// Add a i8 to a i8. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        ~i8::from(self.underlying - ~i8::indent() + other.underlying) // subtract 1 << 7 to avoid double move
    }
}

impl core::ops::Subtract for i8 {
    /// Subtract a i8 from a i8. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~i8::new();
        if self > other {
            res = ~i8::from(self.underlying - other.underlying + ~i8::indent()); // add 1 << 7 to avoid loosing the move
        } else {
            res = ~i8::from(~i8::indent() - (other.underlying - self.underlying)); // subtract from 1 << 7 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for i8 {
    /// Multiply a i8 with a i8. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~i8::new();
        if self.underlying >= ~i8::indent() && other.underlying >= ~i8::indent() {
            res = ~i8::from((self.underlying - ~i8::indent()) * (other.underlying -~i8::indent()) + ~i8::indent());
        } else if self.underlying < ~i8::indent() && other.underlying < ~i8::indent() {
            res = ~i8::from((~i8::indent() - self.underlying) * (~i8::indent() - other.underlying) + ~i8::indent());
        } else if self.underlying >= ~i8::indent() && other.underlying < ~i8::indent() {
            res = ~i8::from(~i8::indent() - (self.underlying - ~i8::indent()) * (~i8::indent() - other.underlying));
        } else if self.underlying < ~i8::indent() && other.underlying >= ~i8::indent() {
            res = ~i8::from(~i8::indent() - (other.underlying - ~i8::indent()) * (~i8::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for i8 {
    /// Divide a i8 by a i8. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        let mut res = ~i8::new();
        if self.underlying >= ~i8::indent() && divisor.underlying >= ~i8::indent() {
            res = ~i8::from((self.underlying - ~i8::indent()) / (divisor.underlying -~i8::indent()) + ~i8::indent());
        } else if self.underlying < ~i8::indent() && divisor.underlying < ~i8::indent() {
            res = ~i8::from((~i8::indent() - self.underlying) / (~i8::indent() - divisor.underlying) + ~i8::indent());
        } else if self.underlying >= ~i8::indent() && divisor.underlying < ~i8::indent() {
            res = ~i8::from(~i8::indent() - (self.underlying - ~i8::indent()) / (~i8::indent() - divisor.underlying));
        } else if self.underlying < ~i8::indent() && divisor.underlying >= ~i8::indent() {
            res = ~i8::from(~i8::indent() - (divisor.underlying - ~i8::indent()) / (~i8::indent() - self.underlying));
        }
        res
    }
}
