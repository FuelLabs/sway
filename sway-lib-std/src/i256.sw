library i256;

use core::num::*;
use ::assert::assert;
use ::u256::U256;

/// The 128-bit signed integer type.
/// Represented as an underlying U256 value.
/// Max value is 2 ^ 127 - 1, min value is - 2 ^ 127
pub struct I256 {
    underlying: U256,
}

pub trait From {
    /// Function for creating I256 from U256
    fn from(value: U256) -> Self;
}

impl From for I256 {
    fn from(value: U256) -> I256 {
        I256 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for I256 {
    fn eq(self, other: I256) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for I256 {
    fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl I256 {
    pub fn indent() -> U256 {
        U256 {
            a: 0,
            b: 1,
            c: 1,
            d: 0,
        }
    }
}

impl I256 {
    /// Initializes a new, zeroed I256.
    pub fn new() -> I256 {
        I256 {
            underlying: ~I256::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> I256 {
        I256 {
            underlying: ~U256::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> I256 {
        I256 {
            underlying: ~U256::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        128
    }

    pub fn neg_from(value: U256) -> I256 {
        I256 {
            underlying: ~I256::indent() - value,
        }
    }

    fn from_uint(value: U256) -> I256 {
        // as the minimal value of I256 is -~I256::indent() (1 << 63) we should add ~I256::indent() (1 << 63) 
        let underlying: U256 = value + ~I256::indent();
        I256 {
            underlying
        }
    }
}

impl core::ops::Add for I256 {
    /// Add a I256 to a I256. Panics on overflow.
    fn add(self, other: Self) -> Self {
        // subtract 1 << 63 to avoid double move
        ~I256::from(self.underlying - ~I256::indent() + other.underlying)
    }
}

impl core::ops::Subtract for I256 {
    /// Subtract a I256 from a I256. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        let mut res = ~I256::new();
        if self > other {
            // add 1 << 63 to avoid loosing the move
            res = ~I256::from(self.underlying - other.underlying + ~I256::indent());
        } else {
            // subtract from 1 << 63 as we are getting a negative value
            res = ~I256::from(~I256::indent() - (other.underlying - self.underlying));
        }
        res
    }
}

impl core::ops::Multiply for I256 {
    /// Multiply a I256 with a I256. Panics of overflow.
    fn multiply(self, other: Self) -> Self {
        let mut res = ~I256::new();
        if (self.underlying > ~I256::indent() || self.underlying == ~I256::indent()) && (other.underlying > ~I256::indent() || other.underlying == ~I256::indent()) {
            res = ~I256::from((self.underlying - ~I256::indent()) * (other.underlying -~I256::indent()) + ~I256::indent());
        } else if self.underlying < ~I256::indent() && other.underlying < ~I256::indent() {
            res = ~I256::from((~I256::indent() - self.underlying) * (~I256::indent() - other.underlying) + ~I256::indent());
        } else if (self.underlying > ~I256::indent() || self.underlying == ~I256::indent()) && other.underlying < ~I256::indent() {
            res = ~I256::from(~I256::indent() - (self.underlying - ~I256::indent()) * (~I256::indent() - other.underlying));
        } else if self.underlying < ~I256::indent() && (other.underlying > ~I256::indent() || other.underlying == ~I256::indent()) {
            res = ~I256::from(~I256::indent() - (other.underlying - ~I256::indent()) * (~I256::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for I256 {
    /// Divide a I256 by a I256. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~I256::new());
        let mut res = ~I256::new();
        if (self.underlying > ~I256::indent() || self.underlying == ~I256::indent()) && divisor.underlying > ~I256::indent() {
            res = ~I256::from((self.underlying - ~I256::indent()) / (divisor.underlying -~I256::indent()) + ~I256::indent());
        } else if self.underlying < ~I256::indent() && divisor.underlying < ~I256::indent() {
            res = ~I256::from((~I256::indent() - self.underlying) / (~I256::indent() - divisor.underlying) + ~I256::indent());
        } else if (self.underlying > ~I256::indent() || self.underlying == ~I256::indent()) && divisor.underlying < ~I256::indent() {
            res = ~I256::from(~I256::indent() - (self.underlying - ~I256::indent()) / (~I256::indent() - divisor.underlying));
        } else if self.underlying < ~I256::indent() && divisor.underlying > ~I256::indent() {
            res = ~I256::from(~I256::indent() - (~I256::indent() - self.underlying) / (divisor.underlying - ~I256::indent()));
        }
        res
    }
}
