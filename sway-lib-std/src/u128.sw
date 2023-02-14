library u128;

use ::assert::assert;
use ::convert::From;
use ::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
use ::math::*;
use ::result::Result;

/// The 128-bit unsigned integer type.
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct U128 {
    upper: u64,
    lower: u64,
}

pub enum U128Error {
    LossOfPrecision: (),
}

impl From<(u64, u64)> for U128 {
    fn from(components: (u64, u64)) -> U128 {
        U128 {
            upper: components.0,
            lower: components.1,
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
    /// Initializes a new, zeroed `U128`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// let new_u128 = U128::new();
    /// let zero_u128 = U128 { upper: 0, lower: 0 };
    ///
    /// assert(new_u128 == zero_u128);
    /// ```
    pub fn new() -> U128 {
        U128 {
            upper: 0,
            lower: 0,
        }
    }

    /// Safely downcast to `u64` without loss of precision.
    /// Returns `Err` if the `number > u64::max()`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::u128::{U128, U128Error};
    ///
    /// let zero_u128 = U128 { upper: 0, lower: 0 };
    /// let zero_u64 = zero_u128.as_u64().unwrap();
    ///
    /// assert(zero_u64 == 0);
    ///
    /// let max_u128 = U128::max();
    /// let result = max_u128.as_u64();
    ///
    /// assert(result.is_err()));
    /// ```
    pub fn as_u64(self) -> Result<u64, U128Error> {
        match self.upper {
            0 => Result::Ok(self.lower),
            _ => Result::Err(U128Error::LossOfPrecision),
        }
    }

    /// The smallest value that can be represented by this integer type.
    /// Initializes a new, zeroed `U128`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// let min_u128 = U128::min();
    /// let zero_u128 = U128 { upper: 0, lower: 0 };
    ///
    /// assert(min_u128 == zero_u128);
    /// ```
    pub fn min() -> U128 {
        U128 {
            upper: 0,
            lower: 0,
        }
    }

    /// The largest value that can be represented by this type,
    /// `2<sup>128</sup> - 1`.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// let max_u128 = U128::max();
    /// let maxed_u128 = U128 { upper: u64::max(), lower: u64::max() };
    ///
    /// assert(max_u128 == maxed_u128);
    /// ```
    pub fn max() -> U128 {
        U128 {
            upper: u64::max(),
            lower: u64::max(),
        }
    }

    /// The size of this type in bits.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// let bits = U128::bits();
    ///
    /// assert(bits == 128);
    /// ```
    pub fn bits() -> u32 {
        128
    }
}

impl core::ops::BitwiseAnd for U128 {
    fn binary_and(self, other: Self) -> Self {
        U128::from((self.upper & other.upper, self.lower & other.lower))
    }
}

impl core::ops::BitwiseOr for U128 {
    fn binary_or(self, other: Self) -> Self {
        U128::from((self.upper | other.upper, self.lower | other.lower))
    }
}

impl core::ops::Shift for U128 {
    fn lsh(self, rhs: u64) -> Self {
        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if rhs >= 128 {
            return Self::new();
        }

        // If shifting by at least half the number of bits, then upper word can
        // be discarded.
        if rhs >= 64 {
            return Self::from((self.lower << (rhs - 64), 0));
        }

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.
        // Save highest bits of lower half.
        let highest_lower_bits = self.lower >> (64 - rhs);

        let upper = (self.upper << rhs) + highest_lower_bits;
        let lower = self.lower << rhs;

        Self::from((upper, lower))
    }

    fn rsh(self, rhs: u64) -> Self {
        // If shifting by at least the number of bits, then saturate with
        // zeroes.
        if (rhs >= 128) {
            return Self::new();
        }

        // If shifting by at least half the number of bits, then lower word can
        // be discarded.
        if (rhs >= 64) {
            return Self::from((0, self.upper >> (rhs - 64)));
        }

        // If shifting by less than half the number of bits, then need to
        // partially shift both upper and lower.
        // Save lowest bits of upper half.
        let lowest_upper_bits = self.upper << (64 - rhs);

        let upper = self.upper >> rhs;
        let lower = (self.lower >> rhs) + lowest_upper_bits;

        Self::from((upper, lower))
    }
}

impl core::ops::Not for U128 {
    fn not(self) -> Self {
        Self {
            upper: !self.upper,
            lower: !self.lower,
        }
    }
}

impl core::ops::Add for U128 {
    /// Add a `U128` to a `U128`. Panics on overflow.
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
    /// Subtract a `U128` from a `U128`. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        // If trying to subtract a larger number, panic.
        assert(!(self < other));

        let mut upper = self.upper - other.upper;
        let mut lower = 0;

        // If necessary, borrow and carry for lower subtraction
        if self.lower < other.lower {
            lower = u64::max() - (other.lower - self.lower - 1);
            upper -= 1;
        } else {
            lower = self.lower - other.lower;
        }

        U128 { upper, lower }
    }
}
impl core::ops::Multiply for U128 {
    /// Multiply a `U128` with a `U128`. Panics of overflow.
    fn multiply(self, other: Self) -> Self {
        // in case both of the `U128` upper parts are bigger than zero,
        // it automatically means overflow, as any `U128` value
        // is upper part multiplied by 2 ^ 64 + lower part
        assert(self.upper == 0 || other.upper == 0);

        let mut result = self.lower.overflowing_mul(other.lower);
        if self.upper == 0 {
            // panic in case of overflow
            result.upper += self.lower * other.upper;
        } else if other.upper == 0 {
            // panic in case of overflow
            result.upper += self.upper * other.lower;
        }

        result
    }
}

impl core::ops::Divide for U128 {
    /// Divide a `U128` by a `U128`. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        let zero = U128::from((0, 0));

        assert(divisor != zero);

        if self.upper == 0 && divisor.upper == 0 {
            return U128::from((0, self.lower / divisor.lower));
        }

        let mut quotient = U128::new();
        let mut remainder = U128::new();
        let mut i = 128 - 1;
        while true {
            quotient <<= 1;
            remainder <<= 1;
            remainder.lower = remainder.lower | (self >> i).lower & 1;
            // TODO use >= once OrdEq can be implemented.
            if remainder > divisor || remainder == divisor {
                remainder -= divisor;
                quotient.lower = quotient.lower | 1;
            }

            if i == 0 {
                break;
            }

            i -= 1;
        }

        quotient
    }
}

impl Power for U128 {
    fn pow(self, exponent: Self) -> Self {
        let mut value = self;
        let mut exp = exponent;
        let one = U128::from((0, 1));
        let zero = U128::from((0, 0));

        if exp == zero {
            return one;
        }

        if exp == one {
            return self;
        }

        while exp & one == zero {
            value = value * value;
            exp >>= 1;
        }

        if exp == one {
            return value;
        }

        let mut acc = value;
        while exp > one {
            exp >>= 1;
            value = value * value;
            if exp & one == one {
                acc = acc * value;
            }
        }
        acc
    }
}

impl Root for U128 {
    /// Integer square root using [Newton's Method](https://en.wikipedia.org/wiki/Integer_square_root#Algorithm_using_Newton's_method).
    fn sqrt(self) -> Self {
        let zero = U128::from((0, 0));
        let mut x0 = self >> 1;
        let mut s = self;

        if x0 != zero {
            let mut x1 = (x0 + s / x0) >> 1;

            while x1 < x0 {
                x0 = x1;
                x1 = (x0 + self / x0) >> 1;
            }

            return x0;
        } else {
            return s;
        }
    }
}

impl BinaryLogarithm for U128 {
    /// `log2` of `x` is the largest `n` such that `2^n <= x < 2^(n+1)`.
    ///
    /// * If `x` is smaller than `2^64`, we could just rely on the `log` method by setting
    /// the base to 2.
    /// * Otherwise, we can find the highest non-zero bit by taking the regular log of the upper
    /// part of the `U128`, and then add 64.
    fn log2(self) -> Self {
        let zero = U128::from((0, 0));
        let mut res = zero;
        // If trying to get a log2(0), panic, as infinity is not a number.
        assert(self != zero);
        if self.upper != 0 {
            res = U128::from((0, self.upper.log(2) + 64));
        } else if self.lower != 0 {
            res = U128::from((0, self.lower.log(2)));
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
