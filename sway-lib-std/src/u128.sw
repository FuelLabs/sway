library u128;

use core::num::*;
use ::assert::assert;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct U128 {
    upper: u64,
    lower: u64,
}

pub trait From {
    /// Function for creating U128 from its u64 components.
    pub fn from(h: u64, l: u64) -> Self;
} {
    fn into(v: U128) -> (u64, u64) {
        (v.upper, v.lower)
    }
}

impl From for U128 {
    pub fn from(h: u64, l: u64) -> U128 {
        U128 {
            upper: h,
            lower: l,
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
        self.upper > other.upper || self.upper == other.upper && self.lower > self.lower
    }

    pub fn lt(self, other: Self) -> bool {
        self.upper < other.upper || self.upper == other.upper && self.lower < self.lower
    }
}

// TODO this doesn't work?
// impl core::ops::OrdEq for U128 {
// }

fn disable_overflow() {
    asm(r1) {
        movi r1 i3;
        flag r1;
    }
}

fn enable_overflow() {
    asm(r1) {
        movi r1 i0;
        flag r1;
    }
}

impl u64 {
    pub fn overflowing_add(a: u64, b: u64) -> U128 {
        disable_overflow();
        let mut v = U128 {
            upper: 0,
            lower: 0,
        };
        asm(r1, r2, a: a, b: b, v_ptr: v) {
            add r1 a b;
            move r2 of;
            sw v_ptr r2 i0;
            sw v_ptr r1 i1;
        };
        enable_overflow();
        v
    }

    pub fn overflowing_mul(self, b: Self) -> U128 {
        disable_overflow();
        let mut v = U128 {
            upper: 0,
            lower: 0,
        };
        asm(r1, r2, a: self, b: b, v_ptr: v) {
            mul r1 a b;
            move r2 of;
            sw v_ptr r2 i0;
            sw v_ptr r1 i1;
        };
        enable_overflow();
        v
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

    /// Divide self by a 64-bit number. Err if result cannot fit in 64 bits, Ok
    /// otherwise.
    pub fn divide_by_u64(self, other: u64) -> Result<u64, ()> {
        // If the upper 64 bits aren't smaller than the divisor, then cannot fit.
        if self.upper >= other {
            return Result::Err(());
        }

        // TODO implement
        let div_lower = self.lower / other;

        return Result::Ok(42);
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

impl core::ops::Add for U128 {
    // Add a U128 to a U128. Panics on overflow.
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
    // Subtract a U128 from a U128. Panics of overflow.
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
            upper: upper,
            lower: lower,
        }
    }
}

impl core::ops::Shiftable for U128 {
    pub fn lsh(self, other: u64) -> Self {
        let mut ret = ~Self::new();

        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if (other >= 128) {
            return ret;
        };

        // If shifting by at least half the number of bits, then upper word can
        // be discarded.
        if (other >= 64) {
            ret.upper = self.lower << (other - 64);
            return ret;
        };

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.

        // Save highest bits of lower half.
        let highest_lower_bits = self.lower >> (64 - other);

        ret.upper = self.upper << other + highest_lower_bits;
        ret.lower = self.lower << other;

        ret
    }

    pub fn rsh(self, other: u64) -> Self {
        let mut ret = ~Self::new();

        // If shifting by at least half the number of bits, then lower word can
        // be discarded.
        if (other >= 64) {
            ret.lower = self.upper >> (other - 64);
            return ret;
        };

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.

        // Save lowest bits of upper half.
        let lowest_upper_bits = self.upper << (64 - other);

        ret.upper = self.upper >> other;
        ret.lower = self.lower >> other + lowest_upper_bits;

        ret
    }
}
