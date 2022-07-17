library i64;

use core::num::*;
use ::assert::assert;
use ::flags::*;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct i64 {
    non_negative: bool,
    module: u32,
}

pub trait From {
    /// Function for creating i64 from its u32 and bool components.
    pub fn from(non_negative: bool, module: u32) -> Self;
}

impl From for i64 {
    pub fn from(non_negative: bool, module: u32) -> i64 {
        i64 {
            non_negative, module,
        }
    }
}

impl core::ops::Eq for i64 {
    pub fn eq(self, other: i64) -> bool {
        self.non_negative == other.non_negative && self.module == other.module
    }
}

impl core::ops::Ord for i64 {
    pub fn gt(self, other: Self) -> bool {
        self.module < other.module && !self.non_negative && !other.non_negative || self.module > other.module && self.non_negative && other.non_negative || self.non_negative && !other.non_negative
    }

    pub fn lt(self, other: Self) -> bool {
        self.module > other.module && !self.non_negative && !other.non_negative || self.module < other.module && self.non_negative && other.non_negative || !self.non_negative && other.non_negative
    }
}

impl i64 {
    /// Initializes a new, zeroed i64.
    pub fn new() -> i64 {
        i64 {
            non_negative: true,
            module: 0,
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i64 {
        i64 {
            non_negative: false,
            module: ~u32::max(),
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>128</sup> - 1.
    pub fn max() -> i64 {
        i64 {
            non_negative: true,
            module: ~u32::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        16
    }
}

impl core::ops::BitwiseAnd for i64 {
    pub fn binary_and(self, other: Self) -> Self {
        ~i64::from(self.non_negative & other.non_negative, self.module & other.module)
    }
}

impl core::ops::BitwiseOr for i64 {
    pub fn binary_or(self, other: Self) -> Self {
        ~i64::from(self.non_negative | other.non_negative, self.module | other.module)
    }
}

impl core::ops::Add for i64 {
    /// Add a i64 to a i64. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        let res = ~i64::new();

        if self.non_negative && other.non_negative {
            res = ~i64::from(true, self.module + other.module);
        } else if !self.non_negative && !other.non_negative {
            res = ~i64::from(false, self.module + other.module);
        } else if !self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i64::from(false, self.module - other.module);
            } else { 
                res = ~i64::from(true, other.module - self.module);
            }
        } else if self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i64::from(true, self.module - other.module);
            } else {
                res = ~i64::from(false, other.module - self.module);
            }
        }
        return res;
    }
}

impl core::ops::Subtract for i64 {
    /// Subtract a i64 from a i64. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let res = ~i64::new();

        if self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i64::from(true, self.module - other.module);
            } else {
                res = ~i64::from(false, other.module - self.module);
            }
        } else if !self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i64::from(false, self.module + other.module);
            } else {
                res = ~i64::from(true, other.module - self.module);
            }
        } else if !self.non_negative && other.non_negative {
            res = ~i64::from(false, other.module + self.module);
        } else if self.non_negative && !other.non_negative {
            res = ~i64::from(true, other.module + self.module);
        }
        return res;
    }
}

impl core::ops::Multiply for i64 {
    /// Multiply a i64 with a i64. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        ~i64::from(self.non_negative & other.non_negative, self.module * other.module)
    }
}

impl core::ops::Divide for i64 {
    /// Divide a i64 by a i64. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        ~i64::from(self.non_negative & other.non_negative, self.module / other.module)
    }
}
