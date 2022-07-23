library i128;

use core::num::*;
use ::assert::assert;
use ::u128::U128;

/// The 128-bit signed integer type.
/// Represented as an underlying U128 value.
pub struct i128 {
    underlying: U128,
}

pub trait From {
    /// Function for creating i128 from its U128 and bool components.
    fn from(value: U128) -> Self;
}

impl From for i128 {
    fn from(value: U128) -> i128 {
        i128 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for i128 {
    pub fn eq(self, other: i128) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for i128 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl i128 {
    pub fn indent() -> U128 {
        U128 {
            upper: 1,
            lower: 0,
        }
    }
}

impl i128 {
    /// Initializes a new, zeroed i128.
    pub fn new() -> i128 {
        i128 {
            underlying: ~i128::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i128 {
        i128 {
            underlying: ~U128::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> i128 {
        i128 {
            underlying: ~U128::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        128
    }

    pub fn neg_from(value: U128) -> i128 {
        i128 {
            underlying: ~i128::indent() - value,
        }
    }

    fn from_uint(value: U128) -> i128 {
        let underlying: U128 = value + ~i128::indent(); // as the minimal value of i128 is -~i128::indent() (1 << 63) we should add ~i128::indent() (1 << 63) 
        i128 {
            underlying
        }
    }
}

impl core::ops::Add for i128 {
    /// Add a i128 to a i128. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        ~i128::from(self.underlying - ~i128::indent() + other.underlying) // subtract 1 << 63 to avoid double move
    }
}

impl core::ops::Subtract for i128 {
    /// Subtract a i128 from a i128. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~i128::new();
        if self > other {
            res = ~i128::from(self.underlying - other.underlying + ~i128::indent()); // add 1 << 63 to avoid loosing the move
        } else {
            res = ~i128::from(~i128::indent() - (other.underlying - self.underlying)); // subtract from 1 << 63 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for i128 {
    /// Multiply a i128 with a i128. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~i128::new();
        if (self.underlying > ~i128::indent() || self.underlying == ~i128::indent()) && (other.underlying > ~i128::indent() || other.underlying == ~i128::indent()) {
            res = ~i128::from((self.underlying - ~i128::indent()) * (other.underlying -~i128::indent()) + ~i128::indent());
        } else if self.underlying < ~i128::indent() && other.underlying < ~i128::indent() {
            res = ~i128::from((~i128::indent() - self.underlying) * (~i128::indent() - other.underlying) + ~i128::indent());
        } else if (self.underlying > ~i128::indent() || self.underlying == ~i128::indent()) && other.underlying < ~i128::indent() {
            res = ~i128::from(~i128::indent() - (self.underlying - ~i128::indent()) * (~i128::indent() - other.underlying));
        } else if self.underlying < ~i128::indent() && (other.underlying > ~i128::indent() || other.underlying == ~i128::indent()) {
            res = ~i128::from(~i128::indent() - (other.underlying - ~i128::indent()) * (~i128::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for i128 {
    /// Divide a i128 by a i128. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        assert(divisor != ~i128::new());
        let mut res = ~i128::new();
        if (self.underlying > ~i128::indent() || self.underlying == ~i128::indent()) && divisor.underlying > ~i128::indent() {
            res = ~i128::from((self.underlying - ~i128::indent()) / (divisor.underlying -~i128::indent()) + ~i128::indent());
        } else if self.underlying < ~i128::indent() && divisor.underlying < ~i128::indent() {
            res = ~i128::from((~i128::indent() - self.underlying) / (~i128::indent() - divisor.underlying) + ~i128::indent());
        } else if (self.underlying > ~i128::indent() || self.underlying == ~i128::indent()) && divisor.underlying < ~i128::indent() {
            res = ~i128::from(~i128::indent() - (self.underlying - ~i128::indent()) / (~i128::indent() - divisor.underlying));
        } else if self.underlying < ~i128::indent() && divisor.underlying > ~i128::indent() {
            res = ~i128::from(~i128::indent() - (~i128::indent() - self.underlying) / (divisor.underlying - ~i128::indent()));
        }
        res
    }
}
