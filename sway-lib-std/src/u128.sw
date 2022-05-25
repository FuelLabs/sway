library u128;

use core::num::*;
use ::assert::assert;
use ::context::registers::flags;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct U128 {
    upper: u64,
    lower: u64,
}

pub trait From {
    /// Function for creating U128 from its u64 components.
    pub fn from(upper: u64, lower: u64) -> Self;
} {
    fn into(val: U128) -> (u64, u64) {
        (val.upper, val.lower)
    }
}

impl From for U128 {
    pub fn from(upper: u64, lower: u64) -> U128 {
        U128 {
            upper, lower,
        }
    }
}

impl core::ops::Eq for U128 {
    pub fn eq(self, other: Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

impl core::ops::Ord for U128 {
    pub fn gt(self, other: Self) -> bool {
        self.upper > other.upper || self.upper == other.upper && self.lower > other.lower
    }

    pub fn lt(self, other: Self) -> bool {
        self.upper < other.upper || self.upper == other.upper && self.lower < other.lower
    }
}

// TODO this doesn't work?
// impl core::ops::OrdEq for U128 {
// }

fn disable_overflow() {
    // Mask second bit, which is `F_WRAPPING`.
    // 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010
    let mask = 2;
    // Get the current value of the flags register and mask it, setting the
    // masked bit. Flags are inverted, so set = off.
    let flag_val = flags() | mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

fn enable_overflow() {
    // Mask second bit, which is `F_WRAPPING`.
    // 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111101
    let mask = 18446744073709551613;
    // Get the current value of the flags register and mask it, unsetting the
    // masked bit. Flags are inverted, so unset = on.
    let flag_val = flags() & mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

impl u64 {
    pub fn overflowing_add(self, right: Self) -> U128 {
        disable_overflow();
        let mut result = U128 {
            upper: 0,
            lower: 0,
        };
        asm(sum, overflow, left: self, right: right, result_ptr: result) {
            add sum left right;
            move overflow of;
            sw result_ptr overflow i0;
            sw result_ptr sum i1;
        };
        enable_overflow();
        result
    }

    pub fn overflowing_mul(self, right: Self) -> U128 {
        disable_overflow();
        let mut result = U128 {
            upper: 0,
            lower: 0,
        };
        asm(product, overflow, left: self, right: right, result_ptr: result) {
            mul product left right;
            move overflow of;
            sw result_ptr overflow i0;
            sw result_ptr product i1;
        };
        enable_overflow();
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

    /// Downcast to `u64`. Err if precision would be lost, Ok otherwise.
    pub fn to_u64(self) -> Result<u64, ()> {
        match self.upper {
            0 => {
                Result::Ok(self.lower)
            },
            _ => {
                Result::Err(())
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
    pub fn binary_and(self, other: Self) -> Self {
        ~U128::from(self.upper & other.upper, self.lower & other.lower)
    }
}

impl core::ops::BitwiseOr for U128 {
    pub fn binary_or(self, other: Self) -> Self {
        ~U128::from(self.upper | other.upper, self.lower | other.lower)
    }
}

impl core::ops::Shiftable for U128 {
    pub fn lsh(self, rhs: u64) -> Self {
        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if (rhs >= 128) {
            return ~Self::new();
        };

        // If shifting by at least half the number of bits, then upper word can
        // be discarded.
        if (rhs >= 64) {
            return ~Self::from(self.lower << (rhs - 64), 0);
        };

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.

        // Save highest bits of lower half.
        let highest_lower_bits = self.lower >> (64 - rhs);

        let upper = (self.upper << rhs) + highest_lower_bits;
        let lower = self.lower << rhs;

        ~Self::from(upper, lower)
    }

    pub fn rsh(self, rhs: u64) -> Self {
        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if (rhs >= 128) {
            return ~Self::new();
        };

        // If shifting by at least half the number of bits, then lower word can
        // be discarded.
        if (rhs >= 64) {
            return ~Self::from(0, self.upper >> (rhs - 64));
        };

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.

        // Save lowest bits of upper half.
        let lowest_upper_bits = self.upper << (64 - rhs);

        let upper = self.upper >> rhs;
        let lower = (self.lower >> rhs) + lowest_upper_bits;

        ~Self::from(upper, lower)
    }
}

impl core::ops::Add for U128 {
    /// Add a U128 to a U128. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        let mut upper_128 = self.upper.overflowing_add(other.upper);

        // If the upper overflows, then the number cannot fit in 128 bits, so panic.
        assert(upper_128.upper == 0);
        let lower_128 = self.lower.overflowing_add(other.lower);

        // If overflow has occurred in the lower component addition, carry.
        // Note: carry can be at most 1.
        if lower_128.upper > 0 {
            upper_128 = upper_128.lower.overflowing_add(lower_128.upper);
        };

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
    pub fn subtract(self, other: Self) -> Self {
        // If trying to subtract a larger number, panic.
        assert(!(self < other));

        let mut upper = self.upper - other.upper;
        let mut lower = 0;

        // If necessary, borrow and carry for lower subtraction
        if self.lower < other.lower {
            lower = ~u64::max() - (other.lower - self.lower - 1);
            upper = upper - 1;
        } else {
            lower = self.lower - other.lower;
        };

        U128 {
            upper, lower,
        }
    }
}

impl core::ops::Multiply for U128 {
    /// Multiply a U128 with a U128. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {
        let zero = ~U128::from(0, 0);
        let one = ~U128::from(0, 1);

        let mut total = ~U128::new();
        // The algorithm loops <from number of bits - 1> to <zero>.
        // Need to add 1 here to invalidate the while loop once i == 0 since we
        // don't have a break keyword.
        let mut i = 128 - 1 + 1;

        while i > 0 {
            // Workaround for not having break keyword
            let shift = i - 1;
            total = total << 1;
            if (other & (one << shift)) != zero {
                total = total + self;
            }

            i = i - 1;
        }

        total
    }
}

impl core::ops::Divide for U128 {
    /// Divide a U128 by a U128. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        let zero = ~U128::from(0, 0);
        let one = ~U128::from(0, 1);

        assert(divisor != zero);

        let mut quotient = ~U128::new();
        let mut remainder = ~U128::new();
        // The algorithm loops <from number of bits - 1> to <zero>.
        // Need to add 1 here to invalidate the while loop once i == 0 since we
        // don't have a break keyword.
        let mut i = 128 - 1 + 1;

        while i > 0 {
            // Workaround for not having break keyword
            let shift = i - 1;
            quotient = quotient << 1;
            remainder = remainder << 1;
            remainder = remainder | ((self & (one << shift)) >> shift);
            // TODO use >= once OrdEq can be implemented.
            if remainder > divisor || remainder == divisor {
                remainder = remainder - divisor;
                quotient = quotient | one;
            }

            i = i - 1;
        }

        quotient
    }
}
