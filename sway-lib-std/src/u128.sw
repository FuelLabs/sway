library u128;

use core::num::*;

use ::assert::assert;
use ::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
use ::result::Result;
use ::math::*;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct U128 {
    upper: u64,
    lower: u64,
}

pub enum U128Error {
    LossOfPrecision: (),
}

pub trait From {
    /// Function for creating U128 from its u64 components.
    fn from(upper: u64, lower: u64) -> Self;
    fn into(self) -> (u64, u64);
}

impl From for U128 {
    fn from(upper: u64, lower: u64) -> U128 {
        U128 {
            upper, lower, 
        }
    }

    fn into(self) -> (u64, u64) {
        (self.upper, self.lower)
    }
}

impl core::ops::Eq for U128 {
    fn eq(self, other: Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

impl core::ops::Ord for U128 {
    fn gt(self, other: Self) -> bool {
        self.upper > other.upper || self.upper == other.upper && self.lower > other.lower
    }

    fn lt(self, other: Self) -> bool {
        self.upper < other.upper || self.upper == other.upper && self.lower < other.lower
    }
}

// TODO this doesn't work?
// impl core::ops::OrdEq for U128 {
// }

impl u64 {
    pub fn overflowing_add(self, right: Self) -> U128 {
        disable_panic_on_overflow();
        let mut result = U128 {
            upper: 0,
            lower: 0,
        };
        asm(sum, overflow, left: self, right: right, result_ptr: result) {
            // Add left and right.
            add sum left right;
            // Immediately copy the overflow of the addition from `$of` into
            // `overflow` so that it's not lost.
            move overflow of;
            // Store the overflow into the first word of result.
            sw result_ptr overflow i0;
            // Store the sum into the second word of result.
            sw result_ptr sum i1;
        };
        enable_panic_on_overflow();
        result
    }

    pub fn overflowing_mul(self, right: Self) -> U128 {
        disable_panic_on_overflow();
        let mut result = U128 {
            upper: 0,
            lower: 0,
        };
        asm(product, overflow, left: self, right: right, result_ptr: result) {
            // Multiply left and right.
            mul product left right;
            // Immediately copy the overflow of the multiplication from `$of` into
            // `overflow` so that it's not lost.
            move overflow of;
            // Store the overflow into the first word of result.
            sw result_ptr overflow i0;
            // Store the product into the second word of result.
            sw result_ptr product i1;
        };
        enable_panic_on_overflow();
        result
    }
}

impl U128 {
    /// Initializes a new, zeroed U128.
    pub fn new() -> U128 {
        U128 {
            upper: 0,
            lower: 0,
        }
    }

    /// Safely downcast to `u64` without loss of precision.
    /// Returns Err if the number > ~u64::max()
    pub fn as_u64(self) -> Result<u64, U128Error> {
        match self.upper {
            0 => {
                Result::Ok(self.lower)
            },
            _ => {
                Result::Err(U128Error::LossOfPrecision)
            },
        }
    }

    /// The smallest value that can be represented by this integer type.
    pub fn min() -> U128 {
        U128 {
            upper: 0,
            lower: 0,
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>128</sup> - 1.
    pub fn max() -> U128 {
        U128 {
            upper: ~u64::max(),
            lower: ~u64::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        128
    }
}

impl core::ops::BitwiseAnd for U128 {
    fn binary_and(self, other: Self) -> Self {
        ~U128::from(self.upper & other.upper, self.lower & other.lower)
    }
}

impl core::ops::BitwiseOr for U128 {
    fn binary_or(self, other: Self) -> Self {
        ~U128::from(self.upper | other.upper, self.lower | other.lower)
    }
}

impl core::ops::Shiftable for U128 {
    fn lsh(self, rhs: u64) -> Self {
        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if rhs >= 128 {
            return ~Self::new();
        }

        // If shifting by at least half the number of bits, then upper word can
        // be discarded.
        if rhs >= 64 {
            return ~Self::from(self.lower <<(rhs - 64), 0);
        }

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.

        // Save highest bits of lower half.
        let highest_lower_bits = self.lower >>(64 - rhs);

        let upper = (self.upper << rhs) + highest_lower_bits;
        let lower = self.lower << rhs;

        ~Self::from(upper, lower)
    }

    fn rsh(self, rhs: u64) -> Self {
        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if (rhs >= 128) {
            return ~Self::new();
        }

        // If shifting by at least half the number of bits, then lower word can
        // be discarded.
        if (rhs >= 64) {
            return ~Self::from(0, self.upper >>(rhs - 64));
        }

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.

        // Save lowest bits of upper half.
        let lowest_upper_bits = self.upper <<(64 - rhs);

        let upper = self.upper >> rhs;
        let lower = (self.lower >> rhs) + lowest_upper_bits;

        ~Self::from(upper, lower)
    }
}

impl core::ops::Add for U128 {
    /// Add a U128 to a U128. Panics on overflow.
    fn add(self, other: Self) -> Self {
        let mut upper_128 = self.upper.overflowing_add(other.upper);

        // If the upper overflows, then the number cannot fit in 128 bits, so panic.
        assert(upper_128.upper == 0);
        let lower_128 = self.lower.overflowing_add(other.lower);

        // If overflow has occurred in the lower component addition, carry.
        // Note: carry can be at most 1.
        if lower_128.upper > 0 {
            upper_128 = upper_128.lower.overflowing_add(lower_128.upper);
        }

        // If overflow has occurred in the upper component addition, panic.
        assert(upper_128.upper == 0);

        U128 {
            upper: upper_128.lower,
            lower: lower_128.lower,
        }
    }
}

impl core::ops::Subtract for U128 {
    /// Subtract a U128 from a U128. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        // If trying to subtract a larger number, panic.
        assert(!(self < other));

        let mut upper = self.upper - other.upper;
        let mut lower = 0;

        // If necessary, borrow and carry for lower subtraction
        if self.lower < other.lower {
            lower = ~u64::max() - (other.lower - self.lower - 1);
            upper -= 1;
        } else {
            lower = self.lower - other.lower;
        }

        U128 {
            upper, lower, 
        }
    }
}

impl core::ops::Multiply for U128 {
    /// Multiply a U128 with a U128. Panics of overflow.
    fn multiply(self, other: Self) -> Self {
        let zero = ~U128::from(0, 0);
        let one = ~U128::from(0, 1);

        let mut total = ~U128::new();
        let mut i = 128 - 1;
        while true {
            total <<= 1;
            if (other & (one << i)) != zero {
                total = total + self;
            }

            if i == 0 {
                break;
            }

            i -= 1;
        }

        total
    }
}

impl core::ops::Divide for U128 {
    /// Divide a U128 by a U128. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        let zero = ~U128::from(0, 0);
        let one = ~U128::from(0, 1);

        assert(divisor != zero);

        let mut quotient = ~U128::new();
        let mut remainder = ~U128::new();
        let mut i = 128 - 1;
        while true {
            quotient <<= 1;
            remainder <<= 1;
            remainder = remainder | ((self & (one << i)) >> i);
            // TODO use >= once OrdEq can be implemented.
            if remainder > divisor || remainder == divisor {
                remainder -= divisor;
                quotient = quotient | one;
            }

            if i == 0 {
                break;
            }

            i -= 1;
        }

        quotient
    }
}

impl Root for U128 {
    fn sqrt(self) -> Self {
        let zero = ~U128::from(0, 0);
        let two = ~U128::from(0, 2);
        let mut x0 = self / two;
        let mut s = self;

        if x0 != zero {
            let mut x1 = (x0 + s / x0) / two;

            while x1 < x0 {
                x0 = x1;
                x1 = (x0 + self / x0) / two;
            }

            return x0;
        } else {
            return s;
        }
    }
}

impl Exponentiate for U128 {
    pub fn pow(self, exponent: Self) -> Self {
        let mut s = self;
        let mut exp: U128 = exponent;
        let one: U128 = ~U128::from(0, 1);
        let zero: U128 = ~U128::from(0, 0);

        if exp == zero {
            return one;
        }

        while exp & one == zero {
            s = s * s;
            exp >>= 1;
        }

        if exp == one {
            return s;
        }

        let mut acc = s;
        while exp > one {
            exp >>= 1;
            s = s * s;
            if exp & one == one {
                acc = acc * s;
            }
        }
        acc
    }
}

impl BinaryLogarithm for U128 {
    pub fn log2(self) -> Self {
        let zero = ~U128::from(0, 0);
        let one = ~U128::from(0, 1);
        let mut res = zero;
        let mut s = self;
        // If trying to get a log2(0), panic, due to infinity not existing.
        assert(!(self == zero));
        while s > zero {
            res += one;
            s >>= 1;
        }
        res
    }
}

impl Logarithm for U128 {
    fn log(self, base: Self) -> Self {
        let self_log2 = self.log2();
        let base_log2 = base.log2();
        self_log2 / base_log2
    }
}
