//! Utilities for common math operations.
library;

/// Calculates the square root.
pub trait Root {
    fn sqrt(self) -> Self;
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

#[test]
fn u256_pow_tests() {
    let five = 0x0000000000000000000000000000000000000000000000000000000000000005u256;

    use ::assert::*;
    
    // 5^2 = 25 = 0x19
    assert_eq(five.pow(2), 0x0000000000000000000000000000000000000000000000000000000000000019u256);

    // 5^28 = 0x204FCE5E3E2502611 (see https://www.wolframalpha.com/input?i=convert+5%5E28+in+hex)
    assert_eq(five.pow(28), 0x0000000000000000204FCE5E3E2502611u256);
}