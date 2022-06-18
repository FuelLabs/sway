library fixed_point;
//! A wrapper around U128 type for a library for Sway for mathematical functions operating with signed 64.64-bit fixed point numbers. 

use ::u128::U128;
use ::assert::assert;

pub struct UFP64 {
    value: U128 
}

impl core::ops::Eq for UFP64 {
    pub fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

impl core::ops::Ord for UFP64 {
    pub fn gt(self, other: Self) -> bool {
        self.value > other.value
    }

    pub fn lt(self, other: Self) -> bool {
        self.value < other.value
    }
}

impl core::ops::Add for UFP64 {
    /// Add a UFP64 to a UFP64. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        UFP64 {
            value: self.value + other.value
        }
    }
}

impl core::ops::Subtract for UFP64 {
    /// Subtract a UFP64 from a UFP64. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        // If trying to subtract a larger number, panic.
        assert(!(self.value < other.value));

        UFP64 {
            value: self.value - other.value
        }
    }
}

impl core::ops::Multiply for UFP64 {
    /// Multiply a UFP64 with a UFP64. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {

        let base = ~U128::from(1,0);

        let self_up = ~U128::from(0, self.value.upper);
        let self_lo = ~U128::from(0, self.value.lower);

        let other_up = ~U128::from(0, other.value.upper);
        let other_lo = ~U128::from(0, other.value.lower);

        let mut up_up = self_up * other_up;
        up_up *= base;
        let mut lo_lo = self_lo * other_lo;
        lo_lo /= base;

        let up_lo = self_up * other_lo;
        let lo_up = self_lo * other_up;

        UFP64 {
            value: up_lo + lo_up
        }
    }
}

impl core::ops::Divide for UFP64 {
    /// Divide a UFP64 by a UFP64. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        let zero = ~UFP64::min();

        assert(divisor != zero);

        let result = self.value / divisor.value;

        UFP64 {
            value: result
        }
    }
}

impl UFP64 {

    /// The smallest value that can be represented by this type.
    pub fn min() -> UFP64 {
        UFP64 {
            value: ~U128::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> UFP64 {
        UFP64 {
            value: ~U128::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        128
    }

    pub fn inv(number: UFP64) -> Self {
        let one = ~U128::from(0,1);
        
        UFP64 {
            value: one / number.value
        }
    }

    pub fn avg(left: UFP64, right: UFP64) -> Self {        
        UFP64 {
            value: (left.value + right.value) / 2,
        }
    }

    pub fn gavg(left: UFP64, right: UFP64) -> Self {        
        UFP64 {
            value: (left.value + right.value) / 2,
        }
    }
}

