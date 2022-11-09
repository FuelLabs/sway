library u256;

use ::assert::assert;
use ::convert::From;
use ::result::Result;
use ::u128::U128;

/// Left shift a u64 and preserve the overflow amount if any
fn lsh_with_carry(word: u64, shift_amount: u64) -> (u64, u64) {
    let right_shift_amount = 64 - shift_amount;
    let carry = word >> right_shift_amount;
    let shifted = word << shift_amount;
    (shifted, carry)
}

/// Right shift a u64 and preserve the overflow amount if any
fn rsh_with_carry(word: u64, shift_amount: u64) -> (u64, u64) {
    let left_shift_amount = 64 - shift_amount;
    let carry = word << left_shift_amount;
    let shifted = word >> shift_amount;
    (shifted, carry)
}

/// The 256-bit unsigned integer type.
/// Represented as four 64-bit components: `(a, b, c, d)`, where `value = (a << 192) + (b << 128) + (c << 64) + d`.
pub struct U256 {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

pub enum U256Error {
    LossOfPrecision: (),
}

impl From<(u64, u64, u64, u64)> for U256 {
    fn from(components: (u64, u64, u64, u64)) -> U256 {
        U256 {
            a: components.0,
            b: components.1,
            c: components.2,
            d: components.3,
        }
    }

    /// Function for extracting 4 u64s from a U256.
    fn into(self) -> (u64, u64, u64, u64) {
        (self.a, self.b, self.c, self.d)
    }
}

impl core::ops::Eq for U256 {
    /// Function for comparing 2 `U256`s for equality
    fn eq(self, other: Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
    }
}

impl U256 {
    /// Initializes a new, zeroed `U256`.
    ///
    /// ### Examples
    /// 
    /// ```sway
    /// use std::u256::U256;
    ///
    /// let new_u256 = U256::new();
    /// let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    ///
    /// assert(new_u256 == zero_u256);
    /// ```
    pub fn new() -> U256 {
        U256 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// Safely downcast to `u64` without loss of precision.
    /// Returns Err if the number > u64::max()
    ///
    /// ### Examples
    /// 
    /// ```sway
    /// use std::u256::{U256, U256Error};
    ///
    /// let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    /// let zero_u64 = zero_u256.as_u64().unwrap();
    ///
    /// assert(zero_u64 == 0);
    ///
    /// let max_u256 = U256::max();
    /// let result = U256.as_u64();
    ///
    /// assert(result == Result::Err(U256Error::LossOfPrecision))
    /// ```
    pub fn as_u64(self) -> Result<u64, U256Error> {
        if self.a == 0 && self.b == 0 && self.c == 0 {
            Result::Ok(self.d)
        } else {
            Result::Err(U256Error::LossOfPrecision)
        }
    }

    /// Safely downcast to `u128` without loss of precision.
    /// Returns an error if `self > U128::max()`.
    ///
    /// ### Examples
    /// 
    /// ```sway
    /// use std::{u128::U128, u256::{U256, U256Error}};
    ///
    /// let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    /// let zero_u128 = zero_u256.as_u128().unwrap();
    ///
    /// assert(zero_u128 == U128 { upper: 0, lower: 0 });
    ///
    /// let max_u256 = U256::max();
    /// let result = U256.as_u64();
    ///
    /// assert(result == Result::Err(U256Error::LossOfPrecision))
    /// ```
    pub fn as_u128(self) -> Result<U128, U256Error> {
        if self.a == 0 && self.b == 0 {
            Result::Ok(U128::from((self.c, self.d)))
        } else {
            Result::Err(U256Error::LossOfPrecision)
        }
    }

    /// The smallest value that can be represented by this integer type.
    ///
    /// ### Examples
    /// 
    /// ```sway
    /// use std::u256::U256;
    ///
    /// let min_u256 = U256::min();
    /// let zero_u256 = U256 { a: 0, b: 0, c: 0, d: 0 };
    ///
    /// assert(min_u256 == zero_u256);
    /// ```
    pub fn min() -> U256 {
        U256 {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>256</sup> - 1.
    ///
    /// ### Examples
    /// 
    /// ```sway
    /// use std::u256::U256;
    ///
    /// let max_u256 = U256::max();
    /// let maxed_u256 = U256 { a: u64::max(), b: u64::max(), c: u64::max(), d: u64::max() };
    ///
    /// assert(max_u256 == maxed_u256);
    /// ```
    pub fn max() -> U256 {
        U256 {
            a: u64::max(),
            b: u64::max(),
            c: u64::max(),
            d: u64::max(),
        }
    }

    /// The size of this type in bits.
    ///
    /// ### Examples
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

    /// Get 4 64 bit words from a single `U256` value.
    ///
    /// ### Examples
    /// 
    /// ```sway
    /// use std::u256::U256;
    ///
    /// let u64s = (1, 2, 3, 4);
    /// let u256: U256::from(u64s);
    /// let decomposed = u256.decompose();
    ///
    /// assert(u64s == decomposed);
    /// ```
    fn decompose(self) -> (u64, u64, u64, u64) {
        (self.a, self.b, self.c, self.d)
    }
}

impl core::ops::Ord for U256 {
    fn gt(self, other: Self) -> bool {
        self.a > other.a || (self.a == other.a && self.b > other.b || (self.b == other.b && self.c > other.c || (self.c == other.c && self.d > other.d)))
    }

    fn lt(self, other: Self) -> bool {
        self.a < other.a || (self.a == other.a && self.b < other.b || (self.b == other.b && self.c < other.c || (self.c == other.c && self.d < other.d)))
    }
}

impl core::ops::BitwiseAnd for U256 {
    fn binary_and(self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = self.decompose();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.decompose();
        let word_1 = value_word_1 & other_word_1;
        let word_2 = value_word_2 & other_word_2;
        let word_3 = value_word_3 & other_word_3;
        let word_4 = value_word_4 & other_word_4;
        U256::from((word_1, word_2, word_3, word_4))
    }
}

impl core::ops::BitwiseOr for U256 {
    fn binary_or(self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = self.decompose();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.decompose();
        let word_1 = value_word_1 | other_word_1;
        let word_2 = value_word_2 | other_word_2;
        let word_3 = value_word_3 | other_word_3;
        let word_4 = value_word_4 | other_word_4;
        U256::from((word_1, word_2, word_3, word_4))
    }
}

impl core::ops::BitwiseXor for U256 {
    fn binary_xor(self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = self.decompose();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.decompose();
        let word_1 = value_word_1 ^ other_word_1;
        let word_2 = value_word_2 ^ other_word_2;
        let word_3 = value_word_3 ^ other_word_3;
        let word_4 = value_word_4 ^ other_word_4;
        U256::from((word_1, word_2, word_3, word_4))
    }
}

impl core::ops::Shiftable for U256 {
    fn lsh(self, shift_amount: u64) -> Self {
        let (word_1, word_2, word_3, word_4) = self.decompose();
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

        U256::from((w1, w2, w3, w4))
    }

    fn rsh(self, shift_amount: u64) -> Self {
        let (word_1, word_2, word_3, word_4) = self.decompose();
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

        U256::from((w1, w2, w3, w4))
    }
}

impl core::ops::Add for U256 {
    /// Add a `U256` to a `U256`. Panics on overflow.
    fn add(self, other: Self) -> Self {
        let (word_1, word_2, word_3, word_4) = self.decompose();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.decompose();

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
        U256::from((result_a, result_b, result_c, result_d))
    }
}

impl core::ops::Subtract for U256 {
    /// Subtract a `U256` from a `U256`. Panics of overflow.
    fn subtract(self, other: Self) -> Self {
        // If trying to subtract a larger number, panic.
        assert(!(self < other));

        let (word_1, word_2, word_3, word_4) = self.decompose();
        let (other_word_1, other_word_2, other_word_3, other_word_4) = other.decompose();

        let mut result_a = word_1 - other_word_1;

        let mut result_b = 0;
        if word_2 < other_word_2 {
            result_b = u64::max() - (other_word_2 - word_2 - 1);
            result_a -= 1;
        } else {
            result_b = word_2 - other_word_2;
        }

        let mut result_c = 0;
        if word_3 < other_word_3 {
            result_c = u64::max() - (other_word_3 - word_3 - 1);
            result_b -= 1;
        } else {
            result_c = word_3 - other_word_3;
        }

        let mut result_d = 0;
        if word_4 < other_word_4 {
            result_d = u64::max() - (other_word_4 - word_4 - 1);
            result_c -= 1;
        } else {
            result_d = word_4 - other_word_4;
        }

        U256::from((result_a, result_b, result_c, result_d))
    }
}

impl core::ops::Multiply for U256 {
    /// Multiply a `U256` with a `U256`. Panics on overflow.
    fn multiply(self, other: Self) -> Self {
        let zero = U256::from((0, 0, 0, 0));
        let one = U256::from((0, 0, 0, 1));

        let mut x = self;
        let mut y = other;
        let mut result = U256::new();
        while y != zero {
            if (y & one).d != 0 {
                result += x;
            }
            x <<= 1;
            y >>= 1;
        }

        result
    }
}

impl core::ops::Divide for U256 {
    /// Divide a `U256` by a `U256`. Panics if divisor is zero.
    fn divide(self, divisor: Self) -> Self {
        let zero = U256::from((0, 0, 0, 0));
        let one = U256::from((0, 0, 0, 1));

        assert(divisor != zero);

        let mut quotient = U256::from((0, 0, 0, 0));
        let mut remainder = U256::from((0, 0, 0, 0));

        let mut i = 256 - 1;

        while true {
            quotient <<= 1;
            remainder <<= 1;

            let m = self & (one << i);
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
