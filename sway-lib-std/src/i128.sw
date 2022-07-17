library i128;

use core::num::*;
use ::assert::assert;
use ::flags::*;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct i128 {
    non_negative: bool,
    module: u64,
}

pub trait From {
    /// Function for creating i128 from its u64 and bool components.
    pub fn from(non_negative: bool, module: u64) -> Self;
}

impl From for i128 {
    pub fn from(non_negative: bool, module: u64) -> i128 {
        i128 {
            non_negative, module,
        }
    }
}

impl core::ops::Eq for i128 {
    pub fn eq(self, other: i128) -> bool {
        self.non_negative == other.non_negative && self.module == other.module
    }
}

impl core::ops::Ord for i128 {
    pub fn gt(self, other: Self) -> bool {
        self.module < other.module && !self.non_negative && !other.non_negative || self.module > other.module && self.non_negative && other.non_negative || self.non_negative && !other.non_negative
    }

    pub fn lt(self, other: Self) -> bool {
        self.module > other.module && !self.non_negative && !other.non_negative || self.module < other.module && self.non_negative && other.non_negative || !self.non_negative && other.non_negative
    }
}

impl i128 {
    /// Initializes a new, zeroed i128.
    pub fn new() -> i128 {
        i128 {
            non_negative: true,
            module: 0,
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i128 {
        i128 {
            non_negative: false,
            module: ~u64::max(),
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>128</sup> - 1.
    pub fn max() -> i128 {
        i128 {
            non_negative: true,
            module: ~u64::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        16
    }
}

impl core::ops::BitwiseAnd for i128 {
    pub fn binary_and(self, other: Self) -> Self {
        ~i128::from(self.non_negative & other.non_negative, self.module & other.module)
    }
}

impl core::ops::BitwiseOr for i128 {
    pub fn binary_or(self, other: Self) -> Self {
        ~i128::from(self.non_negative | other.non_negative, self.module | other.module)
    }
}

impl core::ops::Add for i128 {
    /// Add a i128 to a i128. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        let res = ~i128::new();

        if self.non_negative && other.non_negative {
            res = ~i128::from(true, self.module + other.module);
        } else if !self.non_negative && !other.non_negative {
            res = ~i128::from(false, self.module + other.module);
        } else if !self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i128::from(false, self.module - other.module);
            } else { 
                res = ~i128::from(true, other.module - self.module);
            }
        } else if self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i128::from(true, self.module - other.module);
            } else {
                res = ~i128::from(false, other.module - self.module);
            }
        }
        return res;
    }
}

impl core::ops::Subtract for i128 {
    /// Subtract a i128 from a i128. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let res = ~i128::new();

        if self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i128::from(true, self.module - other.module);
            } else {
                res = ~i128::from(false, other.module - self.module);
            }
        } else if !self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i128::from(false, self.module + other.module);
            } else {
                res = ~i128::from(true, other.module - self.module);
            }
        } else if !self.non_negative && other.non_negative {
            res = ~i128::from(false, other.module + self.module);
        } else if self.non_negative && !other.non_negative {
            res = ~i128::from(true, other.module + self.module);
        }
        return res;
    }
}

impl core::ops::Multiply for i128 {
    /// Multiply a i128 with a i128. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        ~i128::from(self.non_negative & other.non_negative, self.module * other.module)
    }
}

impl core::ops::Divide for i128 {
    /// Divide a i128 by a i128. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        ~i128::from(self.non_negative & other.non_negative, self.module / other.module)
    }
}
