library i16;

use core::num::*;
use ::assert::assert;
use ::flags::*;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct i16 {
    non_negative: bool,
    module: u8,
}

pub trait From {
    /// Function for creating i16 from its u8 and bool components.
    pub fn from(non_negative: bool, module: u8) -> Self;
}

impl From for i16 {
    pub fn from(non_negative: bool, module: u8) -> i16 {
        i16 {
            non_negative, module,
        }
    }
}

impl core::ops::Eq for i16 {
    pub fn eq(self, other: i16) -> bool {
        self.non_negative == other.non_negative && self.module == other.module
    }
}

impl core::ops::Ord for i16 {
    pub fn gt(self, other: Self) -> bool {
        self.module < other.module && !self.non_negative && !other.non_negative || self.module > other.module && self.non_negative && other.non_negative || self.non_negative && !other.non_negative
    }

    pub fn lt(self, other: Self) -> bool {
        self.module > other.module && !self.non_negative && !other.non_negative || self.module < other.module && self.non_negative && other.non_negative || !self.non_negative && other.non_negative
    }
}

impl i16 {
    /// Initializes a new, zeroed i16.
    pub fn new() -> i16 {
        i16 {
            non_negative: true,
            module: 0,
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> i16 {
        i16 {
            non_negative: false,
            module: ~u8::max(),
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>128</sup> - 1.
    pub fn max() -> i16 {
        i16 {
            non_negative: true,
            module: ~u8::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        16
    }
}

impl core::ops::BitwiseAnd for i16 {
    pub fn binary_and(self, other: Self) -> Self {
        ~i16::from(self.non_negative & other.non_negative, self.module & other.module)
    }
}

impl core::ops::BitwiseOr for i16 {
    pub fn binary_or(self, other: Self) -> Self {
        ~i16::from(self.non_negative | other.non_negative, self.module | other.module)
    }
}

impl core::ops::Add for i16 {
    /// Add a i16 to a i16. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        let res = ~i16::new();

        if self.non_negative && other.non_negative {
            res = ~i16::from(true, self.module + other.module);
        } else if !self.non_negative && !other.non_negative {
            res = ~i16::from(false, self.module + other.module);
        } else if !self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i16::from(false, self.module - other.module);
            } else { 
                res = ~i16::from(true, other.module - self.module);
            }
        } else if self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i16::from(true, self.module - other.module);
            } else {
                res = ~i16::from(false, other.module - self.module);
            }
        }
        return res;
    }
}

impl core::ops::Subtract for i16 {
    /// Subtract a i16 from a i16. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        let res = ~i16::new();

        if self.non_negative && other.non_negative {
            if self.module > other.module {
                res = ~i16::from(true, self.module - other.module);
            } else {
                res = ~i16::from(false, other.module - self.module);
            }
        } else if !self.non_negative && !other.non_negative {
            if self.module > other.module {
                res = ~i16::from(false, self.module + other.module);
            } else {
                res = ~i16::from(true, other.module - self.module);
            }
        } else if !self.non_negative && other.non_negative {
            res = ~i16::from(false, other.module + self.module);
        } else if self.non_negative && !other.non_negative {
            res = ~i16::from(true, other.module + self.module);
        }
        return res;
    }
}

impl core::ops::Multiply for i16 {
    /// Multiply a i16 with a i16. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        ~i16::from(self.non_negative & other.non_negative, self.module * other.module)
    }
}

impl core::ops::Divide for i16 {
    /// Divide a i16 by a i16. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        ~i16::from(self.non_negative & other.non_negative, self.module / other.module)
    }
}
