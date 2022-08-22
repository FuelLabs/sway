library i128;

use core::num::*;
use ::assert::assert;
use ::u128::U128;

/// The 128-bit signed integer type.
/// Represented as an underlying U128 value.
/// Max value is 2 ^ 127 - 1, min value is - 2 ^ 127
pub struct I128 {
    underlying: U128,
}

pub trait From {
    /// Function for creating I128 from U128
    fn from(value: U128) -> Self;
}

impl From for I128 {
    fn from(value: U128) -> I128 {
        I128 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for I128 {
    fn eq(self, other: I128) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for I128 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl I128 {
    pub fn indent() -> U128 {
        U128 {
            upper: 1,
            lower: 0,
        }
    }
}

impl I128 {
    /// Initializes a new, zeroed I128.
    pub fn new() -> I128 {
        I128 {
            underlying: ~I128::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> I128 {
        I128 {
            underlying: ~U128::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> I128 {
        I128 {
            underlying: ~U128::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        128
    }

    pub fn neg_from(value: U128) -> I128 {
        I128 {
            underlying: ~I128::indent() - value,
        }
    }

    fn from_uint(value: U128) -> I128 {
        // as the minimal value of I128 is -~I128::indent() (1 << 63) we should add ~I128::indent() (1 << 63) 
        let underlying: U128 = value + ~I128::indent();
        I128 {
            underlying
        }
    }
}

impl core::ops::Add for I128 {
    /// Add a I128 to a I128. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        // subtract 1 << 63 to avoid double move
        ~I128::from(self.underlying - ~I128::indent() + other.underlying)
    }
}

impl core::ops::Subtract for I128 {
    /// Subtract a I128 from a I128. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~I128::new();
        if self > other {
            // add 1 << 63 to avoid loosing the move
            res = ~I128::from(self.underlying - other.underlying + ~I128::indent());
        } else {
            // subtract from 1 << 63 as we are getting a negative value
            res = ~I128::from(~I128::indent() - (other.underlying - self.underlying));
        }
        res
    }
}

impl core::ops::Multiply for I128 {
    /// Multiply a I128 with a I128. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~I128::new();
        if (self.underlying > ~I128::indent() || self.underlying == ~I128::indent()) && (other.underlying > ~I128::indent() || other.underlying == ~I128::indent()) {
            res = ~I128::from((self.underlying - ~I128::indent()) * (other.underlying -~I128::indent()) + ~I128::indent());
        } else if self.underlying < ~I128::indent() && other.underlying < ~I128::indent() {
            res = ~I128::from((~I128::indent() - self.underlying) * (~I128::indent() - other.underlying) + ~I128::indent());
        } else if (self.underlying > ~I128::indent() || self.underlying == ~I128::indent()) && other.underlying < ~I128::indent() {
            res = ~I128::from(~I128::indent() - (self.underlying - ~I128::indent()) * (~I128::indent() - other.underlying));
        } else if self.underlying < ~I128::indent() && (other.underlying > ~I128::indent() || other.underlying == ~I128::indent()) {
            res = ~I128::from(~I128::indent() - (other.underlying - ~I128::indent()) * (~I128::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for I128 {
    /// Divide a I128 by a I128. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~I128::new());
        let mut res = ~I128::new();
        if (self.underlying > ~I128::indent() || self.underlying == ~I128::indent()) && divisor.underlying > ~I128::indent() {
            res = ~I128::from((self.underlying - ~I128::indent()) / (divisor.underlying -~I128::indent()) + ~I128::indent());
        } else if self.underlying < ~I128::indent() && divisor.underlying < ~I128::indent() {
            res = ~I128::from((~I128::indent() - self.underlying) / (~I128::indent() - divisor.underlying) + ~I128::indent());
        } else if (self.underlying > ~I128::indent() || self.underlying == ~I128::indent()) && divisor.underlying < ~I128::indent() {
            res = ~I128::from(~I128::indent() - (self.underlying - ~I128::indent()) / (~I128::indent() - divisor.underlying));
        } else if self.underlying < ~I128::indent() && divisor.underlying > ~I128::indent() {
            res = ~I128::from(~I128::indent() - (~I128::indent() - self.underlying) / (divisor.underlying - ~I128::indent()));
        }
        res
    }
}
