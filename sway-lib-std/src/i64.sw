library i64;

use core::num::*;
use ::assert::assert;

/// The 8-bit signed integer type.
/// Represented as an underlying u64 value.
pub struct i64 {
    underlying: u64,
}

pub trait From {
    /// Function for creating i64 from its u64 and bool components.
    fn from(value: u64) -> Self;
}

impl From for i64 {
    fn from(value: u64) -> i64 {
        i64 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for i64 {
    pub fn eq(self, other: i64) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for i64 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl i64 {
    pub fn indent() -> u64 {
        9223372036854775808u64
    }
}

impl i64 {
    /// Initializes a new, zeroed i64.
    pub fn new() -> i64 {
        i64 {
            underlying: ~i64::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i64 {
        i64 {
            underlying: ~u64::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> i64 {
        i64 {
            underlying: ~u64::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        64
    }

    pub fn neg_from(value: u64) -> i64 {
        i64 {
            underlying: ~i64::indent() - value,
        }
    }

    fn from_uint(value: u64) -> i64 {
        let underlying: u64 = value + ~i64::indent(); // as the minimal value of i64 is -~i64::indent() (1 << 63) we should add ~i64::indent() (1 << 63) 
        i64 {
            underlying
        }
    }
}

impl core::ops::Add for i64 {
    /// Add a i64 to a i64. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        ~i64::from(self.underlying - ~i64::indent() + other.underlying) // subtract 1 << 63 to avoid double move
    }
}

impl core::ops::Subtract for i64 {
    /// Subtract a i64 from a i64. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~i64::new();
        if self > other {
            res = ~i64::from(self.underlying - other.underlying + ~i64::indent()); // add 1 << 63 to avoid loosing the move
        } else {
            res = ~i64::from(~i64::indent() - (other.underlying - self.underlying)); // subtract from 1 << 63 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for i64 {
    /// Multiply a i64 with a i64. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~i64::new();
        if self.underlying >= ~i64::indent() && other.underlying >= ~i64::indent() {
            res = ~i64::from((self.underlying - ~i64::indent()) * (other.underlying -~i64::indent()) + ~i64::indent());
        } else if self.underlying < ~i64::indent() && other.underlying < ~i64::indent() {
            res = ~i64::from((~i64::indent() - self.underlying) * (~i64::indent() - other.underlying) + ~i64::indent());
        } else if self.underlying >= ~i64::indent() && other.underlying < ~i64::indent() {
            res = ~i64::from(~i64::indent() - (self.underlying - ~i64::indent()) * (~i64::indent() - other.underlying));
        } else if self.underlying < ~i64::indent() && other.underlying >= ~i64::indent() {
            res = ~i64::from(~i64::indent() - (other.underlying - ~i64::indent()) * (~i64::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for i64 {
    /// Divide a i64 by a i64. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        let mut res = ~i64::new();
        if self.underlying >= ~i64::indent() && divisor.underlying >= ~i64::indent() {
            res = ~i64::from((self.underlying - ~i64::indent()) / (divisor.underlying -~i64::indent()) + ~i64::indent());
        } else if self.underlying < ~i64::indent() && divisor.underlying < ~i64::indent() {
            res = ~i64::from((~i64::indent() - self.underlying) / (~i64::indent() - divisor.underlying) + ~i64::indent());
        } else if self.underlying >= ~i64::indent() && divisor.underlying < ~i64::indent() {
            res = ~i64::from(~i64::indent() - (self.underlying - ~i64::indent()) / (~i64::indent() - divisor.underlying));
        } else if self.underlying < ~i64::indent() && divisor.underlying >= ~i64::indent() {
            res = ~i64::from(~i64::indent() - (divisor.underlying - ~i64::indent()) / (~i64::indent() - self.underlying));
        }
        res
    }
}
