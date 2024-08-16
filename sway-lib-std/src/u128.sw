//! A 128-bit unsigned integer type.
library;

use ::assert::assert;
use ::convert::{From, Into};
use ::flags::{disable_panic_on_overflow, set_flags};
use ::math::*;
use ::result::Result::{self, *};

/// The 128-bit unsigned integer type.
///
/// # Additional Information
///
/// Represented as two 64-bit components: `(upper, lower)`, where `value = (upper << 64) + lower`.
pub struct U128 {
    /// The most significant 64 bits of the `U128`.
    upper: u64,
    /// The least significant 64 bits of the `U128`.
    lower: u64,
}

impl From<u8> for U128 {
    /// Converts a `u8` to a `U128`.
    ///
    /// # Returns
    ///
    /// * [U128] - The `U128` representation of the `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let u128_value = U128::from(0u8);
    /// }
    /// ```
    fn from(val: u8) -> Self {
        Self {
            upper: 0,
            lower: val.as_u64(),
        }
    }
}

impl From<u16> for U128 {
    /// Converts a `u16` to a `U128`.
    ///
    /// # Returns
    ///
    /// * [U128] - The `U128` representation of the `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let u128_value = U128::from(0u16);
    /// }
    /// ```
    fn from(val: u16) -> Self {
        Self {
            upper: 0,
            lower: val.as_u64(),
        }
    }
}

impl From<u32> for U128 {
    /// Converts a `u32` to a `U128`.
    ///
    /// # Returns
    ///
    /// * [U128] - The `U128` representation of the `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let u128_value = U128::from(0u32);
    /// }
    /// ```
    fn from(val: u32) -> Self {
        Self {
            upper: 0,
            lower: val.as_u64(),
        }
    }
}

impl From<u64> for U128 {
    /// Converts a `u64` to a `U128`.
    ///
    /// # Returns
    ///
    /// * [U128] - The `U128` representation of the `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let u128_value = U128::from(0u64);
    /// }
    /// ```
    fn from(val: u64) -> Self {
        Self {
            upper: 0,
            lower: val,
        }
    }
}

/// The error type used for `U128` type errors.
pub enum U128Error {
    /// This error occurs when a `U128` is attempted to be downcast to a `u64` and the conversion would result in a loss of precision.
    LossOfPrecision: (),
}

impl From<(u64, u64)> for U128 {
    fn from(components: (u64, u64)) -> Self {
        Self {
            upper: components.0,
            lower: components.1,
        }
    }
}

// NOTE: To import, use the glob operator i.e. `use std::u128::*;`
impl From<U128> for (u64, u64) {
    fn from(val: U128) -> (u64, u64) {
        (val.upper, val.lower)
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

impl core::ops::OrdEq for U128 {}

impl u64 {
    /// Performs addition between two `u64` values, returning a `U128`.
    ///
    /// # Additional Information
    ///
    /// Allows for addition between two `u64` values that would otherwise overflow.
    ///
    /// # Arguments
    ///
    /// * `right`: [u64] - The right-hand side of the addition.
    ///
    /// # Returns
    ///
    /// * [U128] - The result of the addition.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let x = u64::max();
    ///     let y = u64::max();
    ///     let z = x.overflowing_add(y);
    ///
    ///     assert(z == U128::from(1, 18446744073709551614));
    /// }
    /// ```
    pub fn overflowing_add(self, right: Self) -> U128 {
        let prior_flags = disable_panic_on_overflow();

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

        set_flags(prior_flags);

        result
    }

    /// Performs multiplication between two `u64` values, returning a `U128`.
    ///
    /// # Additional Information
    ///
    /// Allows for multiplication between two `u64` values that would otherwise overflow.
    ///
    /// # Arguments
    ///
    /// * `right`: [u64] - The right-hand side of the multiplication.
    ///
    /// # Returns
    ///
    /// * [U128] - The result of the multiplication.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let x = u64::max();
    ///     let y = u64::max();
    ///     let z = x.overflowing_mul(y);
    ///
    ///     assert(z == U128::from(18446744073709551615, 1));
    /// }
    /// ```
    pub fn overflowing_mul(self, right: Self) -> U128 {
        let prior_flags = disable_panic_on_overflow();

        let mut result = U128 {
            upper: 0,
            lower: 0,
        };

        asm(
            product,
            overflow,
            left: self,
            right: right,
            result_ptr: result,
        ) {
            mul product left right;
            move overflow of;
            sw result_ptr overflow i0;
            sw result_ptr product i1;
        };

        set_flags(prior_flags);

        result
    }
}

impl U128 {
    /// Initializes a new, zeroed `U128`.
    ///
    /// # Returns
    ///
    /// * [U128] - A new, zero value `U128`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let new_u128 = U128::new();
    ///     let zero_u128 = U128::from(0, 0);
    ///
    ///     assert(new_u128 == zero_u128);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            upper: 0,
            lower: 0,
        }
    }

    /// Safely downcast to `u64` without loss of precision.
    ///
    /// # Additional Information
    ///
    /// If the `U128` is larger than `u64::max()`, an error is returned.
    ///
    /// # Returns
    ///
    /// * [Result<u64, U128Error>] - The result of the downcast.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::{U128, U128Error};
    ///
    /// fn foo() {
    ///     let zero_u128 = U128::from(0, 0);
    ///     let zero_u64 = zero_u128.as_u64().unwrap();
    ///
    ///     assert(zero_u64 == 0);
    ///
    ///     let max_u128 = U128::max();
    ///     let result = max_u128.as_u64();
    ///
    ///     assert(result.is_err()));
    /// }
    /// ```
    pub fn as_u64(self) -> Result<u64, U128Error> {
        match self.upper {
            0 => Ok(self.lower),
            _ => Err(U128Error::LossOfPrecision),
        }
    }

    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [U128] - The smallest value that can be represented by this integer type, `0`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let min_u128 = U128::min();
    ///     let zero_u128 = U128::from(0, 0);
    ///
    ///     assert(min_u128 == zero_u128);
    /// }
    /// ```
    pub fn min() -> Self {
        Self {
            upper: 0,
            lower: 0,
        }
    }

    /// The largest value that can be represented by this type,
    ///
    /// # Returns
    ///
    /// * [U128] - The largest value that can be represented by this type, `2<sup>128</sup> - 1`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let max_u128 = U128::max();
    ///     let maxed_u128 = U128::from(u64::max(), u64::max());
    ///
    ///     assert(max_u128 == maxed_u128);
    /// }
    /// ```
    pub fn max() -> Self {
        Self {
            upper: u64::max(),
            lower: u64::max(),
        }
    }

    /// The size of this type in bits.
    ///
    /// # Returns
    ///
    /// * [u32] - The size of this type in bits, `128`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let bits = U128::bits();
    ///
    ///     assert(bits == 128);
    /// }
    /// ```
    pub fn bits() -> u32 {
        128
    }

    /// Returns the underlying upper u64 representing the most significant 64 bits of the `U128`.
    ///
    /// # Returns
    ///
    /// * [u64] - The most significant 64 bits of the `U128`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let maxed_u128 = U128::from(u64::max(), u64::min());
    ///
    ///     assert(maxed_u128.upper() == u64::max());
    /// }
    /// ```
    pub fn upper(self) -> u64 {
        self.upper
    }

    /// Returns the underlying lower u64 representing the least significant 64 bits of the `U128`.
    ///
    /// # Returns
    ///
    /// * [u64] - The least significant 64 bits of the `U128`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let maxed_u128 = U128::from(u64::max(), u64::min());
    ///
    ///     assert(maxed_u128.lower() == u64::min());
    /// }
    /// ```
    pub fn lower(self) -> u64 {
        self.lower
    }

    /// Returns the zero value for the `U128` type.
    ///
    /// # Returns
    ///
    /// * [U128] -> The zero value for the `U128` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let zero_u128 = U128::zero();
    ///     assert(zero_u128 == U128::from((0, 0)));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            upper: 0,
            lower: 0,
        }
    }

    /// Returns whether a `U128` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `U128` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let zero_u128 = u128::zero();
    ///     assert(zero_u128.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.upper == 0 && self.lower == 0
    }
}

impl core::ops::BitwiseAnd for U128 {
    fn binary_and(self, other: Self) -> Self {
        Self::from((self.upper & other.upper, self.lower & other.lower))
    }
}

impl core::ops::BitwiseOr for U128 {
    fn binary_or(self, other: Self) -> Self {
        Self::from((self.upper | other.upper, self.lower | other.lower))
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
    /// Add a `U128` to a `U128`. Reverts on overflow.
    fn add(self, other: Self) -> Self {
        let self_parts = (0, 0, self.upper, self.lower);
        let other_parts = (0, 0, other.upper, other.lower);

        let self_u256 = asm(r1: self_parts) {
            r1: u256
        };
        let other_u256 = asm(r1: other_parts) {
            r1: u256
        };

        let res_u256 = self_u256 + other_u256;
        let res_parts = asm(r1: res_u256) {
            r1: (u64, u64, u64, u64)
        };

        //assert(res_parts.0 == 0 && res_parts.1 == 0);

        Self {
            upper: res_parts.2,
            lower: res_parts.3,
        }
    }
}

impl core::ops::Subtract for U128 {
    /// Subtract a `U128` from a `U128`. Reverts of overflow.
    fn subtract(self, other: Self) -> Self {
        let self_parts = (0, 0, self.upper, self.lower);
        let other_parts = (0, 0, other.upper, other.lower);

        let self_u256 = asm(r1: self_parts) {
            r1: u256
        };
        let other_u256 = asm(r1: other_parts) {
            r1: u256
        };

        let res_u256 = self_u256 - other_u256;
        let res_parts = asm(r1: res_u256) {
            r1: (u64, u64, u64, u64)
        };

        //assert(res_parts.0 == 0 && res_parts.1 == 0);

        Self {
            upper: res_parts.2,
            lower: res_parts.3,
        }
    }
}
impl core::ops::Multiply for U128 {
    /// Multiply a `U128` with a `U128`. Reverts of overflow.
    fn multiply(self, other: Self) -> Self {
        let self_parts = (0, 0, self.upper, self.lower);
        let other_parts = (0, 0, other.upper, other.lower);

        let self_u256 = asm(r1: self_parts) {
            r1: u256
        };
        let other_u256 = asm(r1: other_parts) {
            r1: u256
        };

        let res_u256 = self_u256 * other_u256;
        let res_parts = asm(r1: res_u256) {
            r1: (u64, u64, u64, u64)
        };

        //assert(res_parts.0 == 0 && res_parts.1 == 0);

        Self {
            upper: res_parts.2,
            lower: res_parts.3,
        }
    }
}

impl core::ops::Divide for U128 {
    /// Divide a `U128` by a `U128`. Reverts if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        let self_parts = (0, 0, self.upper, self.lower);
        let divisor_parts = (0, 0, divisor.upper, divisor.lower);

        let self_u256 = asm(r1: self_parts) {
            r1: u256
        };
        let divisor_u256 = asm(r1: divisor_parts) {
            r1: u256
        };

        let res_u256 = self_u256 / divisor_u256;
        let res_parts = asm(r1: res_u256) {
            r1: (u64, u64, u64, u64)
        };

        //assert(res_parts.0 == 0 && res_parts.1 == 0);

        Self {
            upper: res_parts.2,
            lower: res_parts.3,
        }
    }
}

impl Power for U128 {
    fn pow(self, exponent: u32) -> Self {
        let mut value = self;
        let mut exp = exponent;

        if exp == 0 {
            return Self::from((0, 1));
        }

        if exp == 1 {
            // Manually clone `self`. Otherwise, we may have a `MemoryOverflow`
            // issue with code that looks like: `x = x.pow(other)`
            return Self::from((self.upper, self.lower));
        }

        while exp & 1 == 0 {
            value = value * value;
            exp >>= 1;
        }

        if exp == 1 {
            return value;
        }

        let mut acc = value;
        while exp > 1 {
            exp >>= 1;
            value = value * value;
            if exp & 1 == 1 {
                acc = acc * value;
            }
        }
        acc
    }
}

impl Root for U128 {
    /// Integer square root using [Newton's Method](https://en.wikipedia.org/wiki/Integer_square_root#Algorithm_using_Newton's_method).
    fn sqrt(self) -> Self {
        let zero = Self::from((0, 0));
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
        let zero = Self::from((0, 0));
        let mut res = zero;
        // If trying to get a log2(0), panic, as infinity is not a number.
        assert(self != zero);
        if self.upper != 0 {
            res = Self::from((0, self.upper.log(2) + 64));
        } else if self.lower != 0 {
            res = Self::from((0, self.lower.log(2)));
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
