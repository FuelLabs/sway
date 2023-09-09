//! A 256-bit unsigned integer type.
library;

use ::assert::assert;
use ::convert::From;
use ::result::Result::{*, self};
use ::u128::U128;

/// Left shift a `u64` and preserve the overflow amount if any.
fn lsh_with_carry(word: u64, shift_amount: u64) -> (u64, u64) {
    let right_shift_amount = 64 - shift_amount;
    let carry = word >> right_shift_amount;
    let shifted = word << shift_amount;
    (shifted, carry)
}

/// Right shift a `u64` and preserve the overflow amount if any.
fn rsh_with_carry(word: u64, shift_amount: u64) -> (u64, u64) {
    let left_shift_amount = 64 - shift_amount;
    let carry = word << left_shift_amount;
    let shifted = word >> shift_amount;
    (shifted, carry)
}

/// The 256-bit unsigned integer type.
///
/// # Additional Information
///
/// Represented as four 64-bit components: `(a, b, c, d)`, where `value = (a << 192) + (b << 128) + (c << 64) + d`.
pub struct U256 {
    /// The most significant 64 bits of the `U256`.
    a: u64,
    /// The 65-128th most significant bits of the `U256`.
    b: u64,
    /// The 129-192nd most significant bits of the `U256`.
    c: u64,
    /// The 193-256th most significant bits of the `U256`.
    d: u64,
}

/// The error type used for `U256` type errors.
pub enum U256Error {
    /// This error occurs when a `U256` is attempted to be downcast to a `u64` or `u128` and the conversion would result in a loss of precision.
    LossOfPrecision: (),
}

impl From<(u64, u64, u64, u64)> for U256 {
    fn from(components: (u64, u64, u64, u64)) -> Self {
        Self {
            a: components.0,
            b: components.1,
            c: components.2,
            d: components.3,
        }
    }

    /// Function for extracting 4 `u64`s from a `U256`.
    fn into(self) -> (u64, u64, u64, u64) {
        (self.a, self.b, self.c, self.d)
    }
}

impl core::ops::Eq for U256 {
    /// Function for comparing 2 `U256`s for equality.
    fn eq(self, other: Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
    }
}

impl U256 {
    /// Initializes a new, zeroed `U256`.
    ///
    /// # Returns
    ///
    /// * [U256] - A new, zero value `U256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u256::U256;
    ///
    /// fn foo() {
    ///     let new_u256 = U256::new();
    ///     let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    ///
    ///     assert(new_u256 == zero_u256);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// Safely downcast to `u64` without loss of precision.
    ///
    /// # Additional Information
    ///
    /// If the `U256` is larger than `u64::max()`, an error will be returned.
    ///
    /// # Returns
    ///
    /// * [Result<u64, U256Error>] - The `U256` as a `u64` or an error if the conversion would result in a loss of precision.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u256::{U256, U256Error};
    ///
    /// fn foo() {
    ///     let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    ///     let zero_u64 = zero_u256.as_u64().unwrap();
    ///
    ///     assert(zero_u64 == 0);
    ///
    ///     let max_u256 = U256::max();
    ///     let result = U256.as_u64();
    ///
    ///     assert(result.is_err()))
    /// }
    /// ```
    pub fn as_u64(self) -> Result<u64, U256Error> {
        if self.a == 0 && self.b == 0 && self.c == 0 {
            Ok(self.d)
        } else {
            Err(U256Error::LossOfPrecision)
        }
    }

    /// Safely downcast to `U128` without loss of precision.
    ///
    /// # Additional Information
    ///
    /// If the `U256` is larger than `U128::max()`, an error will be returned.
    ///
    /// # Returns
    ///
    /// * [Result<u128, U256Error>] - The `U256` as a `U128` or an error if the conversion would result in a loss of precision.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{u128::U128, u256::{U256, U256Error}};
    ///
    /// fn foo() {
    ///     let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    ///     let zero_u128 = zero_u256.as_u128().unwrap();
    ///
    ///     assert(zero_u128 == U128 { upper: 0, lower: 0 });
    ///
    ///     let max_u256 = U256::max();
    ///     let result = U256.as_u64();
    ///
    ///     assert(result.is_err()))
    /// }
    /// ```
    pub fn as_u128(self) -> Result<U128, U256Error> {
        if self.a == 0 && self.b == 0 {
            Ok(U128::from((self.c, self.d)))
        } else {
            Err(U256Error::LossOfPrecision)
        }
    }

    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [U256] - The smallest value that can be represented by this integer type, `0`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u256::U256;
    ///
    /// fn foo() {
    ///     let min_u256 = U256::min();
    ///     let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    ///
    ///     assert(min_u256 == zero_u256);
    /// }
    /// ```
    pub fn min() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// The largest value that can be represented by this type.
    ///
    /// # Returns
    ///
    /// * [U256] - The largest value that can be represented by this type, `2<sup>256</sup> - 1`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u256::U256;
    ///
    /// fn foo() {
    ///     let max_u256 = U256::max();
    ///     let maxed_u256 = U256 { a: u64::max(), b: u64::max(), c: u64::max(), d: u64::max() };
    ///
    ///     assert(max_u256 == maxed_u256);
    /// }
    /// ```
    pub fn max() -> Self {
        Self {
            a: u64::max(),
            b: u64::max(),
            c: u64::max(),
            d: u64::max(),
        }
    }

    /// The size of this type in bits.
    ///
    /// # Returns
    ///
    /// * [u32] - The size of this type in bits, `256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::u256::U256;
    ///
    /// let bits = U256::bits();
    ///
    /// assert(bits == 256);
    /// ```
    pub fn bits() -> u32 {
        256
    }
}

impl core::ops::Ord for U256 {
    fn gt(self, other: Self) -> bool {
        self.a > other.a || (self.a == other.a && (self.b > other.b || (self.b == other.b && (self.c > other.c || (self.c == other.c && self.d > other.d)))))
    }

    fn lt(self, other: Self) -> bool {
        self.a < other.a || (self.a == other.a && (self.b < other.b || (self.b == other.b && (self.c < other.c || (self.c == other.c && self.d < other.d)))))
    }
}

#[test]
fn test_u256_ord() {
    assert(U256::from((0, 0, 0, 1)) < U256::from((0, u64::max(), 0, 0)));
    assert(!(U256::from((0, 0, 0, 1)) > U256::from((0, u64::max(), 0, 0))));

    assert(U256::from((0, u64::max(), 0, 0)) > U256::from((0, 0, 0, 1)));
    assert(!(U256::from((0, u64::max(), 0, 0)) < U256::from((0, 0, 0, 1))));

    assert(U256::max() > U256::from((0, 0, u64::max(), u64::max())));
    assert(!(U256::max() < U256::from((0, 0, u64::max(), u64::max()))));
    assert(U256::from((0, 0, u64::max(), u64::max())) < U256::max());
    assert(!(U256::from((0, 0, u64::max(), u64::max())) > U256::max()));
}

impl core::ops::BitwiseAnd for U256 {
    fn binary_and(self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = self.into();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.into();
        let word_1 = value_word_1 & other_word_1;
        let word_2 = value_word_2 & other_word_2;
        let word_3 = value_word_3 & other_word_3;
        let word_4 = value_word_4 & other_word_4;
        Self::from((word_1, word_2, word_3, word_4))
    }
}

impl core::ops::BitwiseOr for U256 {
    fn binary_or(self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = self.into();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.into();
        let word_1 = value_word_1 | other_word_1;
        let word_2 = value_word_2 | other_word_2;
        let word_3 = value_word_3 | other_word_3;
        let word_4 = value_word_4 | other_word_4;
        Self::from((word_1, word_2, word_3, word_4))
    }
}

impl core::ops::BitwiseXor for U256 {
    fn binary_xor(self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = self.into();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.into();
        let word_1 = value_word_1 ^ other_word_1;
        let word_2 = value_word_2 ^ other_word_2;
        let word_3 = value_word_3 ^ other_word_3;
        let word_4 = value_word_4 ^ other_word_4;
        Self::from((word_1, word_2, word_3, word_4))
    }
}

impl core::ops::Shift for U256 {
    fn lsh(self, shift_amount: u64) -> Self {
        let (word_1, word_2, word_3, word_4) = self.into();
        let mut w1 = 0;
        let mut w2 = 0;
        let mut w3 = 0;
        let mut w4 = 0;

        let w = shift_amount / 64; // num of whole words to shift in addition to b
        let b = shift_amount % 64; // num of bits to shift within each word
        if w == 0 {
            let (shifted_2, carry_2) = lsh_with_carry(word_2, b);
            w1 = (word_1 << b) + carry_2;
            let (shifted_3, carry_3) = lsh_with_carry(word_3, b);
            w2 = shifted_2 + carry_3;
            let (shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w3 = shifted_3 + carry_4;
            w4 = shifted_4;
        } else if w == 1 {
            let (shifted_3, carry_3) = lsh_with_carry(word_3, b);
            w1 = (word_2 << b) + carry_3;
            let (shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w2 = shifted_3 + carry_4;
            w3 = shifted_4;
        } else if w == 2 {
            let (shifted_4, carry_4) = lsh_with_carry(word_4, b);
            w1 = (word_3 << b) + carry_4;
            w2 = shifted_4;
        } else if w == 3 {
            w1 = word_4 << b;
        }

        Self::from((w1, w2, w3, w4))
    }

    fn rsh(self, shift_amount: u64) -> Self {
        let (word_1, word_2, word_3, word_4) = self.into();
        let mut w1 = 0;
        let mut w2 = 0;
        let mut w3 = 0;
        let mut w4 = 0;

        let w = shift_amount / 64; // num of whole words to shift in addition to b
        let b = shift_amount % 64; // num of bits to shift within each word
        if w == 0 {
            let (shifted_3, carry_3) = rsh_with_carry(word_3, b);
            w4 = (word_4 >> b) + carry_3;
            let (shifted_2, carry_2) = rsh_with_carry(word_2, b);
            w3 = shifted_3 + carry_2;
            let (shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w2 = shifted_2 + carry_1;
            w1 = shifted_1;
        } else if w == 1 {
            let (shifted_2, carry_2) = rsh_with_carry(word_2, b);
            w4 = (word_3 >> b) + carry_2;
            let (shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w3 = shifted_2 + carry_1;
            w2 = shifted_1;
        } else if w == 2 {
            let (shifted_1, carry_1) = rsh_with_carry(word_1, b);
            w4 = (word_2 >> b) + carry_1;
            w3 = shifted_1;
        } else if w == 3 {
            w4 = word_1 >> b;
        };

        Self::from((w1, w2, w3, w4))
    }
}

impl core::ops::Not for U256 {
    fn not(self) -> Self {
        Self {
            a: !self.a,
            b: !self.b,
            c: !self.c,
            d: !self.d,
        }
    }
}

impl core::ops::Add for U256 {
    /// Add a `U256` to a `U256`. Reverts on overflow.
    fn add(self, other: Self) -> Self {
        let (word_1, word_2, word_3, word_4) = self.into();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.into();

        let mut overflow = 0;
        let mut local_res = U128::from((0, word_4)) + U128::from((0, other_word_4));
        let result_d = local_res.lower;
        overflow = local_res.upper;

        local_res = U128::from((0, word_3)) + U128::from((0, other_word_3)) + U128::from((0, overflow));
        let result_c = local_res.lower;
        overflow = local_res.upper;

        local_res = U128::from((0, word_2)) + U128::from((0, other_word_2)) + U128::from((0, overflow));
        let result_b = local_res.lower;
        overflow = local_res.upper;

        local_res = U128::from((0, word_1)) + U128::from((0, other_word_1)) + U128::from((0, overflow));
        let result_a = local_res.lower;
        // panic on overflow
        assert(local_res.upper == 0);
        Self::from((result_a, result_b, result_c, result_d))
    }
}

impl core::ops::Subtract for U256 {
    /// Subtract a `U256` from a `U256`. Reverts of overflow.
    fn subtract(self, other: Self) -> Self {
        if self == other {
            return Self::min();
        } else if other == Self::min() {
            // Manually clone `self`. Otherwise, we may have a `MemoryOverflow`
            // issue with code that looks like: `x = x - other`
            return Self::from((self.a, self.b, self.c, self.d));
        }
        // If trying to subtract a larger number, panic.
        assert(self > other);
        let (word_1, word_2, word_3, word_4) = self.into();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.into();

        let mut result_a = word_1 - other_word_1;
        let mut result_b = 0;
        if word_2 < other_word_2 {
            result_b = u64::max() - (other_word_2 - word_2 - 1);
            // we assume that result_a > 0, as in case result_a <= 0 means that lhs of the operation is smaller than rhs,
            // which we ruled out at the beginning of the function.
            result_a -= 1;
        } else {
            result_b = word_2 - other_word_2;
        }
        let mut result_c = 0;
        if word_3 < other_word_3 {
            result_c = u64::max() - (other_word_3 - word_3 - 1);
            if result_b > 0 {
                result_b -= 1;
            } else {
                // we assume that result_a > 0, as in case result_a <= 0 means that lhs of the operation is smaller than rhs,
                // which we ruled out at the beginning of the function.
                result_a -= 1;
                result_b = u64::max();
            }
        } else {
            result_c = word_3 - other_word_3;
        }

        let mut result_d = 0;
        if word_4 < other_word_4 {
            result_d = u64::max() - (other_word_4 - word_4 - 1);
            if result_c > 0 {
                result_c -= 1;
            } else {
                if result_b > 0 {
                    result_b -= 1;
                } else {
                    // we assume that result_a > 0, as in case result_a <= 0 means that lhs of the operation is smaller than rhs,
                    // which we ruled out at the beginning of the function.
                    result_a -= 1;
                    result_b = u64::max();
                }
                result_c = u64::max();
            }
        } else {
            result_d = word_4 - other_word_4;
        }

        Self::from((result_a, result_b, result_c, result_d))
    }
}

impl core::ops::Multiply for U256 {
    /// Multiply a `U256` with a `U256`. Reverts on overflow.
    fn multiply(self, other: Self) -> Self {
        // Both upper words cannot be non-zero simultaneously. Otherwise, overflow is guaranteed.
        assert(self.a == 0 || other.a == 0);

        if self.a != 0 {
            // If `self.a` is non-zero, all words of `other`, except for `d`, should be zero.
            // Otherwise, overflow is guaranteed.
            assert(other.b == 0 && other.c == 0);
            Self::from((self.a * other.d, 0, 0, 0))
        } else if other.a != 0 {
            // If `other.a` is non-zero, all words of `self`, except for `d`, should be zero.
            // Otherwise, overflow is guaranteed.
            assert(self.b == 0 && self.c == 0);
            Self::from((other.a * self.d, 0, 0, 0))
        } else {
            if self.b != 0 {
                // If `self.b` is non-zero, `other.b` has  to be zero. Otherwise, overflow is
                // guaranteed because:
                // `other.b * 2 ^ (64 * 2) * self.b * 2 ^ (62 ^ 2) > 2 ^ (64 * 4)`
                assert(other.b == 0);
                let result_b_d = self.b.overflowing_mul(other.d);
                let result_c_d = self.c.overflowing_mul(other.d);
                let result_d_c = self.d.overflowing_mul(other.c);
                let result_d_d = self.d.overflowing_mul(other.d);

                let (overflow_of_c_to_b_1, mut c) = result_d_d.upper.overflowing_add(result_c_d.lower).into();
                let (mut overflow_of_c_to_b_2, c) = c.overflowing_add(result_d_c.lower).into();

                let (overflow_of_b_to_a_0, overflow_of_c_to_b_2) = overflow_of_c_to_b_1.overflowing_add(overflow_of_c_to_b_2).into();

                let (overflow_of_b_to_a_1, mut b) = result_b_d.lower.overflowing_add(result_c_d.upper).into();
                let (overflow_of_b_to_a_2, b) = b.overflowing_add(result_d_c.upper).into();
                let (overflow_of_b_to_a_3, b) = b.overflowing_add(overflow_of_c_to_b_2).into();

                Self::from((
                    self.b * other.c + result_b_d.upper + overflow_of_b_to_a_3 + overflow_of_b_to_a_2 + overflow_of_b_to_a_1 + overflow_of_b_to_a_0,
                    b,
                    c,
                    result_d_d.lower,
                ))
            } else if other.b != 0 {
                // If `other.b` is nonzero, `self.b` has to be zero. Otherwise, overflow is
                // guaranteed because:
                // `other.b * 2 ^ (64 * 2) * self.b * 2 ^ (62 ^ 2) > 2 ^ (64 * 4)`.
                assert(self.b == 0);
                let result_b_d = other.b.overflowing_mul(self.d);
                let result_c_d = other.c.overflowing_mul(self.d);
                let result_d_c = other.d.overflowing_mul(self.c);
                let result_d_d = other.d.overflowing_mul(self.d);

                let (overflow_of_c_to_b_1, mut c) = result_d_d.upper.overflowing_add(result_c_d.lower).into();
                let (mut overflow_of_c_to_b_2, c) = c.overflowing_add(result_d_c.lower).into();

                let (overflow_of_b_to_a_0, overflow_of_c_to_b_2) = overflow_of_c_to_b_1.overflowing_add(overflow_of_c_to_b_2).into();

                let (overflow_of_b_to_a_1, mut b) = result_b_d.lower.overflowing_add(result_c_d.upper).into();
                let (overflow_of_b_to_a_2, b) = b.overflowing_add(result_d_c.upper).into();
                let (overflow_of_b_to_a_3, b) = b.overflowing_add(overflow_of_c_to_b_2).into();

                Self::from((
                    other.b * self.c + result_b_d.upper + overflow_of_b_to_a_3 + overflow_of_b_to_a_2 + overflow_of_b_to_a_1 + overflow_of_b_to_a_0,
                    b,
                    c,
                    result_d_d.lower,
                ))
            } else {
                // note, that `self.a`, `self.b`, `other.a`, `other.b` are all equal to 0
                let result_c_c = other.c.overflowing_mul(self.c);
                let result_c_d = self.c.overflowing_mul(other.d);
                let result_d_c = self.d.overflowing_mul(other.c);
                let result_d_d = self.d.overflowing_mul(other.d);

                let (overflow_of_c_to_b_1, mut c) = result_d_d.upper.overflowing_add(result_c_d.lower).into();

                let (mut overflow_of_c_to_b_2, c) = c.overflowing_add(result_d_c.lower).into();

                let (overflow_of_b_to_a_0, overflow_of_c_to_b_2) = overflow_of_c_to_b_1.overflowing_add(overflow_of_c_to_b_2).into();

                let (overflow_of_b_to_a_1, mut b) = result_c_c.lower.overflowing_add(result_c_d.upper).into();
                let (overflow_of_b_to_a_2, b) = b.overflowing_add(result_d_c.upper).into();
                let (overflow_of_b_to_a_3, b) = b.overflowing_add(overflow_of_c_to_b_2).into();

                Self::from((
                    // as overflow for a means overflow for the whole number, we are adding as is, not using `overflowing_add`
                    result_c_c.upper + overflow_of_b_to_a_3 + overflow_of_b_to_a_2 + overflow_of_b_to_a_1 + overflow_of_b_to_a_0,
                    b,
                    c,
                    result_d_d.lower,
                ))
            }
        }
    }
}

impl core::ops::Divide for U256 {
    /// Divide a `U256` by a `U256`. Reverts if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        let zero = Self::from((0, 0, 0, 0));
        let one = Self::from((0, 0, 0, 1));

        assert(divisor != zero);

        if self.a == 0
            && self.b == 0
            && divisor.a == 0
            && divisor.b == 0
        {
            let res = U128::from((self.c, self.d)) / U128::from((divisor.c, divisor.d));
            return Self::from((0, 0, res.upper, res.lower));
        }

        let mut quotient = Self::from((0, 0, 0, 0));
        let mut remainder = Self::from((0, 0, 0, 0));

        let mut i = 256 - 1;

        while true {
            quotient <<= 1;
            remainder <<= 1;

            let _m = self & (one << i);
            remainder.d = remainder.d | (self >> i).d & 1;
            // TODO use >= once OrdEq can be implemented.
            if remainder > divisor || remainder == divisor {
                remainder -= divisor;
                quotient.d = quotient.d | 1;
            }

            if i == 0 {
                break;
            }

            i -= 1;
        }

        quotient
    }
}
