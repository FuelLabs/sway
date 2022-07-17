library i32;

use core::num::*;
use ::assert::assert;

/// The 8-bit signed integer type.
/// Represented as an underlying u32 value.
pub struct i32 {
    underlying: u32,
}

pub trait From {
    /// Function for creating i32 from its u32 and bool components.
    fn from(value: u32) -> Self;
}

impl From for i32 {
    fn from(value: u32) -> i32 {
        i32 {
            underlying: value,
        }
    }
}

impl core::ops::Eq for i32 {
    pub fn eq(self, other: i32) -> bool {
        self.underlying == other.underlying
    }
}

impl core::ops::Ord for i32 {
    pub fn gt(self, other: Self) -> bool {
        self.underlying > other.underlying
    }

    pub fn lt(self, other: Self) -> bool {
        self.underlying < other.underlying
    }
}

impl i32 {
    pub fn indent() -> u32 {
        2147483648u32
    }
}

impl i32 {
    /// Initializes a new, zeroed i32.
    pub fn new() -> i32 {
        i32 {
            underlying: ~i32::indent(),
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i32 {
        i32 {
            underlying: ~u32::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> i32 {
        i32 {
            underlying: ~u32::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        32
    }

    pub fn neg_from(value: u32) -> i32 {
        i32 {
            underlying: ~i32::indent() - value,
        }
    }

    fn from_uint(value: u32) -> i32 {
        let underlying: u32 = value + ~i32::indent(); // as the minimal value of i32 is 2147483648 (1 << 31) we should add ~i32::indent() (1 << 31) 
        i32 {
            underlying
        }
    }
}

impl core::ops::Add for i32 {
    /// Add a i32 to a i32. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        ~i32::from(self.underlying - ~i32::indent() + other.underlying) // subtract 1 << 31 to avoid double move
    }
}

impl core::ops::Subtract for i32 {
    /// Subtract a i32 from a i32. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let mut res = ~i32::new();
        if self > other {
            res = ~i32::from(self.underlying - other.underlying + ~i32::indent()); // add 1 << 31 to avoid loosing the move
        } else {
            res = ~i32::from(~i32::indent() - (other.underlying - self.underlying)); // subtract from 1 << 31 as we are getting a negative value
        }
        res
    }
}

impl core::ops::Multiply for i32 {
    /// Multiply a i32 with a i32. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let mut res = ~i32::new();
        if self.underlying >= ~i32::indent() && other.underlying >= ~i32::indent() {
            res = ~i32::from((self.underlying - ~i32::indent()) * (other.underlying -~i32::indent()) + ~i32::indent());
        } else if self.underlying < ~i32::indent() && other.underlying < ~i32::indent() {
            res = ~i32::from((~i32::indent() - self.underlying) * (~i32::indent() - other.underlying) + ~i32::indent());
        } else if self.underlying >= ~i32::indent() && other.underlying < ~i32::indent() {
            res = ~i32::from(~i32::indent() - (self.underlying - ~i32::indent()) * (~i32::indent() - other.underlying));
        } else if self.underlying < ~i32::indent() && other.underlying >= ~i32::indent() {
            res = ~i32::from(~i32::indent() - (other.underlying - ~i32::indent()) * (~i32::indent() - self.underlying));
        }
        res
    }
}

impl core::ops::Divide for i32 {
    /// Divide a i32 by a i32. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        let mut res = ~i32::new();
        if self.underlying >= ~i32::indent() && divisor.underlying >= ~i32::indent() {
            res = ~i32::from((self.underlying - ~i32::indent()) / (divisor.underlying -~i32::indent()) + ~i32::indent());
        } else if self.underlying < ~i32::indent() && divisor.underlying < ~i32::indent() {
            res = ~i32::from((~i32::indent() - self.underlying) / (~i32::indent() - divisor.underlying) + ~i32::indent());
        } else if self.underlying >= ~i32::indent() && divisor.underlying < ~i32::indent() {
            res = ~i32::from(~i32::indent() - (self.underlying - ~i32::indent()) / (~i32::indent() - divisor.underlying));
        } else if self.underlying < ~i32::indent() && divisor.underlying >= ~i32::indent() {
            res = ~i32::from(~i32::indent() - (divisor.underlying - ~i32::indent()) / (~i32::indent() - self.underlying));
        }
        res
    }
}
