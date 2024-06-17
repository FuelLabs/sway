//! Utilities for common math operations.
library;

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

impl Power for u256 {
    /// Raises self to the power of `exponent`, using exponentiation by squaring.
    ///
    /// # Panics
    ///
    /// Panics if the result overflows the type.
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
                acc = acc * base;
            }
            exp = exp >> 1;
            base = base * base;
        }

        acc * base
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
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
        }
    }
}

impl Power for u16 {
    fn pow(self, exponent: u32) -> Self {
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
        }
    }
}

impl Power for u8 {
    fn pow(self, exponent: u32) -> Self {
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
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
        use ::assert::*;
        assert(self != 0);
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
        let self_log2 = self.log2();
        let base_log2 = base.log2();
        self_log2 / base_log2
    }
}

impl u8 {
    /// Returns whether a `u8` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u8` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u8 = u8::zero();
    ///     assert(zero_u8.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u8
    }
}

impl u16 {
    /// Returns whether a `u16` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u16` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u16 = u16::zero();
    ///     assert(zero_u16.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u16
    }
}

impl u32 {
    /// Returns whether a `u32` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u32` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u32 = u32::zero();
    ///     assert(zero_u32.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u32
    }
}

impl u64 {
    /// Returns whether a `u64` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u64` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u64 = u64::zero();
    ///     assert(zero_u64.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u64
    }
}

impl u256 {
    /// Returns whether a `u256` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u256` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u256 = u256::zero();
    ///     assert(zero_u256.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0x00u256
    }
}

impl b256 {
    /// Returns whether a `b256` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `b256` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_b256 = b256::zero();
    ///     assert(zero_b256.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0x0000000000000000000000000000000000000000000000000000000000000000
    }
}
