//! A 128-bit unsigned integer type.
library;

use ::assert::assert;
use ::convert::{From, Into, TryFrom};
use ::flags::{
    disable_panic_on_overflow,
    panic_on_overflow_enabled,
    panic_on_unsafe_math_enabled,
    set_flags,
};
use ::registers::{flags, overflow};
use ::math::*;
use ::result::Result::{self, *};
use ::option::Option::{self, None, Some};
use ::revert::revert;
use ::ops::*;
use ::codec::*;
use ::debug::*;

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

impl PartialEq for U128 {
    fn eq(self, other: Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}
impl Eq for U128 {}

impl Ord for U128 {
    fn gt(self, other: Self) -> bool {
        self.upper > other.upper || self.upper == other.upper && self.lower > other.lower
    }

    fn lt(self, other: Self) -> bool {
        self.upper < other.upper || self.upper == other.upper && self.lower < other.lower
    }
}

impl OrdEq for U128 {}

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

    // TODO: Rename to `try_as_u64` to be consistent with all other downcasts
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

    /// Upcasts a `U128` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `U128` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///     let u128_value = U128::from(0u64);
    ///     let u256_value = u128_value.as_u256();
    /// }
    pub fn as_u256(self) -> u256 {
        asm(nums: (0, 0, self.upper, self.lower)) {
            nums: u256
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

impl BitwiseAnd for U128 {
    fn binary_and(self, other: Self) -> Self {
        Self::from((self.upper & other.upper, self.lower & other.lower))
    }
}

impl BitwiseOr for U128 {
    fn binary_or(self, other: Self) -> Self {
        Self::from((self.upper | other.upper, self.lower | other.lower))
    }
}

impl Shift for U128 {
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

impl Not for U128 {
    fn not(self) -> Self {
        Self {
            upper: !self.upper,
            lower: !self.lower,
        }
    }
}

impl Add for U128 {
    /// Add a `U128` to a `U128`. Reverts on overflow.
    fn add(self, other: Self) -> Self {
        let mut upper_128 = self.upper.overflowing_add(other.upper);

        if panic_on_overflow_enabled() {
            // If the upper overflows, then the number cannot fit in 128 bits, so panic.
            assert(upper_128.upper == 0);
        }

        let lower_128 = self.lower.overflowing_add(other.lower);

        // If overflow has occurred in the lower component addition, carry.
        // Note: carry can be at most 1.
        if lower_128.upper > 0 {
            upper_128 = upper_128.lower.overflowing_add(lower_128.upper);
        }

        if panic_on_overflow_enabled() {
            // If overflow has occurred in the upper component addition, panic.
            assert(upper_128.upper == 0);
        }

        Self {
            upper: upper_128.lower,
            lower: lower_128.lower,
        }
    }
}

impl Subtract for U128 {
    /// Subtract a `U128` from a `U128`. Reverts on underflow.
    fn subtract(self, other: Self) -> Self {
        // panic_on_overflow_enabled is also for underflow
        if panic_on_overflow_enabled() {
            // If trying to subtract a larger number, panic.
            assert(!(self < other));
        }

        let mut upper = self.upper - other.upper;
        let mut lower = 0;

        // If necessary, borrow and carry for lower subtraction
        if self.lower < other.lower {
            lower = u64::max() - (other.lower - self.lower - 1);
            upper -= 1;
        } else {
            lower = self.lower - other.lower;
        }

        Self { upper, lower }
    }
}
impl Multiply for U128 {
    /// Multiply a `U128` with a `U128`. Reverts of overflow.
    fn multiply(self, other: Self) -> Self {
        // in case both of the `U128` upper parts are bigger than zero,
        // it automatically means overflow, as any `U128` value
        // is upper part multiplied by 2 ^ 64 + lower part
        if panic_on_unsafe_math_enabled() {
            assert(self.upper == 0 || other.upper == 0);
        }

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

impl Divide for U128 {
    /// Divide a `U128` by a `U128`. Reverts if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        let zero = Self::from((0, 0));

        if panic_on_unsafe_math_enabled() {
            assert(divisor != zero);
        } else {
            if divisor == zero {
                return zero;
            }
        }

        if self.upper == 0 && divisor.upper == 0 {
            return Self::from((0, self.lower / divisor.lower));
        }

        let mut quotient = Self::new();
        let mut remainder = Self::new();
        let mut i = 128 - 1;
        while true {
            quotient <<= 1;
            remainder <<= 1;
            remainder.lower = remainder.lower | (self >> i).lower & 1;
            if remainder >= divisor {
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

impl Mod for U128 {
    fn modulo(self, other: Self) -> Self {
        if panic_on_unsafe_math_enabled() {
            assert(other != Self::zero());
        }

        // a mod b = a - b * (a / b)
        let quotient = self / other;
        let product = quotient * other;
        self - product
    }
}

fn u64_checked_add(a: u64, b: u64) -> Option<u64> {
    let of = asm(a: a, b: b, res) {
        add res a b;
        of: u64
    };

    if of != 0 {
        return None;
    }

    Some(a + b)
}

fn u128_checked_mul(a: U128, b: U128) -> Option<U128> {
    // in case both of the `U128` upper parts are bigger than zero,
    // it automatically means overflow, as any `U128` value
    // is upper part multiplied by 2 ^ 64 + lower part
    if a.upper != 0 && b.upper != 0 {
        return None
    }

    let mut result = a.lower.overflowing_mul(b.lower);

    if a.upper == 0 {
        match u64_checked_add(result.upper, a.lower * b.upper) {
            None => return None,
            Some(v) => {
                result.upper = v
            }
        }
    } else if b.upper == 0 {
        match u64_checked_add(result.upper, a.upper * b.lower) {
            None => return None,
            Some(v) => {
                result.upper = v
            }
        }
    }

    Some(result)
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
            match u128_checked_mul(value, value) {
                None => {
                    if panic_on_overflow_enabled() {
                        revert(0);
                    } else {
                        // Return zero on overflow as per the Fuel VM Specifications
                        return U128::zero()
                    }
                },
                Some(v) => value = v,
            };
            exp >>= 1;
        }

        if exp == 1 {
            return value;
        }

        let mut acc = value;
        while exp > 1 {
            exp >>= 1;
            match u128_checked_mul(value, value) {
                None => if panic_on_overflow_enabled() {
                    revert(0);
                } else {
                    // Return zero on overflow as per the Fuel VM Specifications
                    return U128::zero()
                },
                Some(v) => value = v,
            };
            if exp & 1 == 1 {
                match u128_checked_mul(acc, value) {
                    None => if panic_on_overflow_enabled() {
                        revert(0);
                    } else {
                        // Return zero on overflow as per the Fuel VM Specifications
                        return U128::zero()
                    },
                    Some(v) => acc = v,
                };
            }
        }
        acc
    }
}

impl Root for U128 {
    /// Integer square root using [Newton's Method](https://en.wikipedia.org/wiki/Integer_square_root#Algorithm_using_Newton's_method).
    fn sqrt(self) -> Self {
        let zero = Self::from((0, 0));
        if panic_on_unsafe_math_enabled() {
            assert(self != zero);
        }

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
        // If panic on unsafe math is enabled, only then revert
        if panic_on_unsafe_math_enabled() {
            assert(self != zero);
        } else {
            if self == zero {
                return zero;
            }
        }
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
        let flags = disable_panic_on_overflow();

        // If panic on unsafe math is enabled, only then revert
        if panic_on_unsafe_math_enabled() {
            // Logarithm is undefined for bases less than 2
            assert(base >= U128::from(2_u64));
            // Logarithm is undefined for 0
            assert(self != U128::zero());
        } else {
            // Logarithm is undefined for bases less than 2
            // Logarithm is undefined for 0
            if (base < U128::from(2_u64)) || (self == U128::zero()) {
                set_flags(flags);
                return U128::zero();
            }
        }

        // Decimals rounded to 0
        if self < base {
            set_flags(flags);
            return U128::zero();
        }

        // Estimating the result using change of base formula. Only an estimate because we are doing uint calculations.
        let self_log2 = self.log2();
        let base_log2 = base.log2();
        let mut result = (self_log2 / base_log2);

        // Converting u128 to u32, this cannot fail as the result will be atmost ~128
        let parts: (u64, u64) = result.into();
        let res_u32 = asm(r1: parts.1) {
            r1: u32
        };

        // Raising the base to the power of the result
        let mut pow_res = base.pow(res_u32);
        let mut of = overflow();

        // Adjusting the result until the power is less than or equal to self
        // If pow_res is > than self, then there is an overestimation. If there is an overflow then there is definitely an overestimation.
        while (pow_res > self) || (of > 0) {
            result -= U128::from(1_u64);

            // Converting u128 to u32, this cannot fail as the result will be atmost ~128
            let parts: (u64, u64) = result.into();
            let res_u32 = asm(r1: parts.1) {
                r1: u32
            };

            pow_res = base.pow(res_u32);
            of = overflow();
        };

        set_flags(flags);

        result
    }
}

impl TotalOrd for U128 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TryFrom<U128> for u8 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            <u8 as TryFrom<u64>>::try_from(u.lower())
        } else {
            None
        }
    }
}

impl TryFrom<U128> for u16 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            <u16 as TryFrom<u64>>::try_from(u.lower())
        } else {
            None
        }
    }
}

impl TryFrom<U128> for u32 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            <u32 as TryFrom<u64>>::try_from(u.lower())
        } else {
            None
        }
    }
}

impl TryFrom<U128> for u64 {
    fn try_from(u: U128) -> Option<Self> {
        if u.upper() == 0 {
            Some(u.lower())
        } else {
            None
        }
    }
}

impl From<U128> for u256 {
    /// Converts a `U128` to a `u256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [U128] - The `U128` to be converted.
    ///
    /// # Returns
    ///
    /// * [u256] - The `u256` representation of the `U128` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///    let u128_value = U128::from((18446744073709551615_u64, 18446744073709551615_u64));
    ///    let u256_value = u256::from(u128_value);
    /// }
    /// ```
    fn from(num: U128) -> Self {
        let input = (0u64, 0u64, num.upper(), num.lower());
        asm(input: input) {
            input: u256
        }
    }
}

impl From<U128> for b256 {
    /// Converts a `U128` to a `b256`.
    ///
    /// # Arguments
    ///
    /// * `num`: [U128] - The `U128` to be converted.
    ///
    /// # Returns
    ///
    /// * [b256] - The `b256` representation of the `U128` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u128::U128;
    ///
    /// fn foo() {
    ///    let u128_value = U128::from((18446744073709551615_u64, 18446744073709551615_u64));
    ///    let b256_value = b256::from(u128_value);
    /// }
    /// ```
    fn from(num: U128) -> Self {
        let input = (0u64, 0u64, num.upper(), num.lower());
        asm(input: input) {
            input: b256
        }
    }
}
