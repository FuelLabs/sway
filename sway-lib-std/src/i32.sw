library i32;

use core::num::*;
use ::assert::assert;
use ::flags::*;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct i32 {
    non_negative: bool,
    module: u16,
}

pub trait From {
    /// Function for creating i32 from its u16 and bool components.
    pub fn from(non_negative: bool, module: u16) -> Self;
}

impl From for i32 {
    pub fn from(non_negative: bool, module: u16) -> i32 {
        i32 {
            non_negative, module,
        }
    }
}

impl core::ops::Eq for i32 {
    pub fn eq(self, other: i32) -> bool {
        self.non_negative == other.non_negative && self.module == other.module
    }
}

impl core::ops::Ord for i32 {
    pub fn gt(self, other: Self) -> bool {
        self.module < other.module && !self.non_negative && !other.non_negative || self.module > other.module && self.non_negative && other.non_negative || self.non_negative && !other.non_negative
    }

    pub fn lt(self, other: Self) -> bool {
        self.module > other.module && !self.non_negative && !other.non_negative || self.module < other.module && self.non_negative && other.non_negative || !self.non_negative && other.non_negative
    }
}

impl i32 {
    /// Initializes a new, zeroed i32.
    pub fn new() -> i32 {
        i32 {
            non_negative: true,
            module: 0,
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i32 {
        i32 {
            non_negative: false,
            module: ~u16::max(),
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>128</sup> - 1.
    pub fn max() -> i32 {
        i32 {
            non_negative: true,
            module: ~u16::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        16
    }
}

impl core::ops::BitwiseAnd for i32 {
    pub fn binary_and(self, other: Self) -> Self {
        ~i32::from(self.non_negative & other.non_negative, self.module & other.module)
    }
}

impl core::ops::BitwiseOr for i32 {
    pub fn binary_or(self, other: Self) -> Self {
        ~i32::from(self.non_negative | other.non_negative, self.module | other.module)
    }
}

impl core::ops::Add for i32 {
    /// Add a i32 to a i32. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        let res = ~i32::new();

        if self.non_negative && other.non_negative {
            res = ~i32::from(true, self.module + other.module);
        } else if !self.non_negative && !other.non_negative {
            res = ~i32::from(false, self.module + other.module);
        } else if !self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i32::from(false, self.module - other.module);
            } else { 
                res = ~i32::from(true, other.module - self.module);
            }
        } else if self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i32::from(true, self.module - other.module);
            } else {
                res = ~i32::from(false, other.module - self.module);
            }
        }
        return res;
    }
}

impl core::ops::Subtract for i32 {
    /// Subtract a i32 from a i32. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let res = ~i32::new();

        if self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i32::from(true, self.module - other.module);
            } else {
                res = ~i32::from(false, other.module - self.module);
            }
        } else if !self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i32::from(false, self.module + other.module);
            } else {
                res = ~i32::from(true, other.module - self.module);
            }
        } else if !self.non_negative && other.non_negative {
            res = ~i32::from(false, other.module + self.module);
        } else if self.non_negative && !other.non_negative {
            res = ~i32::from(true, other.module + self.module);
        }
        return res;
    }
}

impl core::ops::Multiply for i32 {
    /// Multiply a i32 with a i32. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        ~i32::from(self.non_negative & other.non_negative, self.module * other.module)
    }
}

impl core::ops::Divide for i32 {
    /// Divide a i32 by a i32. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        ~i32::from(self.non_negative & other.non_negative, self.module / other.module)
    }
}
