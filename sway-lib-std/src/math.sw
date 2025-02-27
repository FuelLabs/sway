//! Utilities for common math operations.
library;

use ::assert::*;
use ::revert::revert;
use ::option::Option::{self, None, Some};
use ::flags::{
    disable_panic_on_overflow,
    panic_on_overflow_enabled,
    panic_on_unsafe_math_enabled,
    set_flags,
};
use ::registers::{flags, overflow};
use ::primitive_conversions::{u16::*, u256::*, u32::*, u64::*, u8::*};

/// Calculates the square root.
pub trait Root {
    fn sqrt(self) -> Self;
}

impl Root for u256 {
    // Integer square root using [Newton's Method](https://en.wikipedia.org/wiki/Integer_square_root#Algorithm_using_Newton's_method).
    fn sqrt(self) -> Self {
        let mut x0 = self >> 1;
        if x0 == 0 {
            return self;
        }
        let mut x1 = (x0 + self / x0) >> 1;

        while x1 < x0 {
            x0 = x1;
            x1 = (x0 + self / x0) >> 1;
        }

        x0
    }
}

impl Root for u64 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u64
        }
    }
}

impl Root for u32 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u32
        }
    }
}

impl Root for u16 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u16
        }
    }
}

impl Root for u8 {
    fn sqrt(self) -> Self {
        let index: u64 = 2;
        asm(r1: self, r2: index, r3) {
            mroo r3 r1 r2;
            r3: u8
        }
    }
}

/// Calculates a number to a given power.
pub trait Power {
    fn pow(self, exponent: u32) -> Self;
}

fn u256_checked_mul(a: u256, b: u256) -> Option<u256> {
    let res = u256::zero();

    // The six-bit immediate value is used to select operating mode, as follows:

    // Bits	Short name	Description
    // ..XXXX	reserved	Reserved and must be zero
    // .X....	indirect0	Is lhs operand ($rB) indirect or not
    // X.....	indirect1	Is rhs operand ($rC) indirect or not
    // As both operands are indirect, 110000 is used, which is 48 in decimal.
    let of = asm(res: res, a: a, b: b) {
        wqml res a b i48;
        of: u64
    };

    if of != 0 {
        return None;
    }

    Some(res)
}

impl Power for u256 {
    /// Raises self to the power of `exponent`, using exponentiation by squaring.
    ///
    /// # Additional Information
    ///
    /// * If panic on overflow is disabled, and the result overflows, the return value will be 0.
    ///
    /// # Reverts
    ///
    /// * Reverts if the result overflows the type, if panic on overflow is enabled.
    fn pow(self, exponent: u32) -> Self {
        let one = 0x0000000000000000000000000000000000000000000000000000000000000001u256;

        if exponent == 0 {
            return one;
        }

        let mut exp = exponent;
        let mut base = self;
        let mut acc = one;

        while exp > 1 {
            if (exp & 1) == 1 {
                // acc = acc * base;
                let res = u256_checked_mul(acc, base);
                acc = match res {
                    Some(val) => val,
                    None => return u256::zero(),
                }
            }
            exp = exp >> 1;
            // base = base * base;
            let res = u256_checked_mul(base, base);
            base = match res {
                Some(val) => val,
                None => return u256::zero(),
            }
        }

        // acc * base
        let res = u256_checked_mul(acc, base);
        match res {
            Some(val) => val,
            None => u256::zero(),
        }
    }
}

impl Power for u64 {
    fn pow(self, exponent: u32) -> Self {
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
        }
    }
}

impl Power for u32 {
    fn pow(self, exponent: u32) -> Self {
        let mut res = asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: u64
        };

        if res > Self::max().as_u64() {
            // If panic on wrapping math is enabled, only then revert
            if panic_on_overflow_enabled() {
                revert(0);
            } else {
                // Follow spec of returning 0 for overflow
                res = 0;
            }
        }

        asm(r1: res) {
            r1: Self
        }
    }
}

impl Power for u16 {
    fn pow(self, exponent: u32) -> Self {
        let mut res = asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: u64
        };

        if res > Self::max().as_u64() {
            // If panic on wrapping math is enabled, only then revert
            if panic_on_overflow_enabled() {
                revert(0);
            } else {
                // Follow spec of returning 0 for overflow
                res = 0;
            }
        }

        asm(r1: res) {
            r1: Self
        }
    }
}

impl Power for u8 {
    fn pow(self, exponent: u32) -> Self {
        let mut res = asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: u64
        };

        if res > Self::max().as_u64() {
            // If panic on wrapping math is enabled, only then revert
            if panic_on_overflow_enabled() {
                revert(0);
            } else {
                // Follow spec of returning 0 for overflow
                res = 0;
            }
        }

        asm(r1: res) {
            r1: Self
        }
    }
}

/// Trait for exponential functions.
/// This should exist for UFP64, UFP128 and their signed versions.
pub trait Exponent {
    // exponential function: e ^ exponent
    fn exp(exponent: Self) -> Self;
}

/// Calculates the log with a given base.
pub trait Logarithm {
    fn log(self, base: Self) -> Self;
}

impl Logarithm for u64 {
    fn log(self, base: Self) -> Self {
        asm(r1: self, r2: base, r3) {
            mlog r3 r1 r2;
            r3: Self
        }
    }
}

impl Logarithm for u32 {
    fn log(self, base: Self) -> Self {
        asm(r1: self, r2: base, r3) {
            mlog r3 r1 r2;
            r3: Self
        }
    }
}

impl Logarithm for u16 {
    fn log(self, base: Self) -> Self {
        asm(r1: self, r2: base, r3) {
            mlog r3 r1 r2;
            r3: Self
        }
    }
}

impl Logarithm for u8 {
    fn log(self, base: Self) -> Self {
        asm(r1: self, r2: base, r3) {
            mlog r3 r1 r2;
            r3: Self
        }
    }
}

/// Calculates the binary log.
pub trait BinaryLogarithm {
    fn log2(self) -> Self;
}

impl BinaryLogarithm for u64 {
    fn log2(self) -> Self {
        self.log(2)
    }
}

impl BinaryLogarithm for u32 {
    fn log2(self) -> Self {
        self.log(2u32)
    }
}

impl BinaryLogarithm for u16 {
    fn log2(self) -> Self {
        self.log(2u16)
    }
}

impl BinaryLogarithm for u8 {
    fn log2(self) -> Self {
        self.log(2u8)
    }
}

impl BinaryLogarithm for u256 {
    fn log2(self) -> Self {
        // If panic on unsafe math is enabled, only then revert
        if panic_on_unsafe_math_enabled() {
            // Logarithm is undefined for 0
            assert(self != 0);
        }

        let (a, b, c, d) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };
        if a != 0 {
            return a.log2().as_u256() + 0xc0u256;
        } else if b != 0 {
            return b.log2().as_u256() + 0x80u256;
        } else if c != 0 {
            return c.log2().as_u256() + 0x40u256;
        } else if d != 0 {
            return d.log2().as_u256();
        }
        self
    }
}

impl Logarithm for u256 {
    fn log(self, base: Self) -> Self {
        let flags = disable_panic_on_overflow();

        // If panic on unsafe math is enabled, only then revert
        if panic_on_unsafe_math_enabled() {
            // Logarithm is undefined for bases less than 2
            assert(base >= 2);
            // Logarithm is undefined for 0
            assert(self != 0);
        } else {
            // Logarithm is undefined for bases less than 2
            // Logarithm is undefined for 0
            if (base < 2) || (self == 0) {
                set_flags(flags);
                return 0x00u256;
            }
        }

        // Decimals rounded to 0
        if self < base {
            set_flags(flags);
            return 0x00u256;
        }

        // Estimating the result using change of base formula. Only an estimate because we are doing uint calculations.
        let self_log2 = self.log2();
        let base_log2 = base.log2();
        let mut result = (self_log2 / base_log2);

        // Converting u256 to u32, this cannot fail as the result will be atmost ~256
        let parts = asm(r1: result) {
            r1: (u64, u64, u64, u64)
        };
        let res_u32 = asm(r1: parts.3) {
            r1: u32
        };

        // Raising the base to the power of the result
        let mut pow_res = base.pow(res_u32);
        let mut of = overflow();

        // Adjusting the result until the power is less than or equal to self
        // If pow_res is > than self, then there is an overestimation. If there is an overflow then there is definitely an overestimation.
        while (pow_res > self) || (of > 0) {
            result -= 1;

            // Converting u256 to u32, this cannot fail as the result will be atmost ~256
            let parts = asm(r1: result) {
                r1: (u64, u64, u64, u64)
            };
            let res_u32 = asm(r1: parts.3) {
                r1: u32
            };

            pow_res = base.pow(res_u32);
            of = overflow();
        };

        set_flags(flags);

        result
    }
}
