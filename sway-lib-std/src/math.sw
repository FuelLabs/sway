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
fn square_root_test_math_sw() {
    use ::assert::*;

    let max_u64 = u64::max();
    let max_u32 = u32::max();
    let max_u16 = u16::max();
    let max_u8 = u8::max();

    // u64
    assert(1.sqrt() == 1);
    assert(4.sqrt() == 2);
    assert(9.sqrt() == 3);
    assert(144.sqrt() == 12);
    assert(1024.sqrt() == 32);
    assert(10000000000000000.sqrt() == 100000000);
    assert(0.sqrt() == 0);
    assert(2.sqrt() == 1);
    assert(5.sqrt() == 2);
    assert(1000.sqrt() == 31);
    assert(max_u64.sqrt() == 4294967295);

    // u32
    assert(1u32.sqrt() == 1);
    assert(4u32.sqrt() == 2);
    assert(9u32.sqrt() == 3);
    assert(144u32.sqrt() == 12);
    assert(1024u32.sqrt() == 32);
    assert(100000000u32.sqrt() == 10000);
    assert(0u32.sqrt() == 0);
    assert(2u32.sqrt() == 1);
    assert(5u32.sqrt() == 2);
    assert(1000u32.sqrt() == 31);
    assert(max_u32.sqrt() == 65535);

    // u16
    assert(1u16.sqrt() == 1);
    assert(4u16.sqrt() == 2);
    assert(9u16.sqrt() == 3);
    assert(144u16.sqrt() == 12);
    assert(1024u16.sqrt() == 32);
    assert(50625u16.sqrt() == 225);
    assert(0u16.sqrt() == 0);
    assert(2u16.sqrt() == 1);
    assert(5u16.sqrt() == 2);
    assert(1000u16.sqrt() == 31);
    assert(max_u16.sqrt() == 255);

    // u8
    assert(1u8.sqrt() == 1);
    assert(4u8.sqrt() == 2);
    assert(9u8.sqrt() == 3);
    assert(144u8.sqrt() == 12);
    assert(0u8.sqrt() == 0);
    assert(2u8.sqrt() == 1);
    assert(5u8.sqrt() == 2);
    assert(max_u8.sqrt() == 15);
}

#[test]
fn exponentiation_test_math_sw() {
    use ::assert::*;

    // u256
    
    let five = 0x0000000000000000000000000000000000000000000000000000000000000005u256;

    // 5^2 = 25 = 0x19
    assert_eq(
        five
            .pow(2),
        0x0000000000000000000000000000000000000000000000000000000000000019u256,
    );

    // 5^28 = 0x204FCE5E3E2502611 (see https://www.wolframalpha.com/input?i=convert+5%5E28+in+hex)
    assert_eq(five.pow(28), 0x0000000000000000204FCE5E3E2502611u256);

    // u64
    assert(2.pow(2) == 4);
    assert(2 ** 2 == 4);

    assert(2.pow(3) == 8);
    assert(2 ** 3 == 8);

    assert(42.pow(2) == 1764);
    assert(42 ** 2 == 1764);

    assert(42.pow(3) == 74088);
    assert(42 ** 3 == 74088);

    assert(100.pow(5) == 10000000000);
    assert(100 ** 5 == 10000000000);

    assert(100.pow(8) == 10000000000000000);
    assert(100 ** 8 == 10000000000000000);

    assert(100.pow(9) == 1000000000000000000);
    assert(100 ** 9 == 1000000000000000000);

    assert(2.pow(0) == 1);
    assert(2 ** 0 == 1);

    assert(0.pow(1) == 0);
    assert(0 ** 1 == 0);

    assert(0.pow(2) == 0);
    assert(0 ** 2 == 0);

    // u32
    assert(2u32.pow(2u32) == 4u32);
    assert(2u32 ** 2u32 == 4u32);

    assert(2u32.pow(3u32) == 8u32);
    assert(2u32 ** 3u32 == 8u32);

    assert(42u32.pow(2u32) == 1764u32);
    assert(42u32 ** 2u32 == 1764u32);

    assert(100u32.pow(4u32) == 100000000u32);
    assert(100u32 ** 4u32 == 100000000u32);

    assert(2u32.pow(0u32) == 1u32);
    assert(2u32 ** 0u32 == 1u32);

    assert(0u32.pow(1u32) == 0u32);
    assert(0u32 ** 1u32 == 0u32);

    assert(0u32.pow(2u32) == 0u32);
    assert(0u32 ** 2u32 == 0u32);

    // u16
    assert(2u16.pow(2u32) == 4u16);
    assert(2u16 ** 2u32 == 4u16);

    assert(2u16.pow(3u32) == 8u16);
    assert(2u16 ** 3u32 == 8u16);

    assert(42u16.pow(2u32) == 1764u16);
    assert(42u16 ** 2u32 == 1764u16);

    assert(20u16.pow(3u32) == 8000u16);
    assert(20u16 ** 3u32 == 8000u16);

    assert(15u16.pow(4u32) == 50625u16);
    assert(15u16 ** 4u32 == 50625u16);

    assert(2u16.pow(0u32) == 1u16);
    assert(2u16 ** 0u32 == 1u16);

    assert(0u16.pow(1u32) == 0u16);
    assert(0u16 ** 1u32 == 0u16);

    assert(0u16.pow(2u32) == 0u16);
    assert(0u16 ** 2u32 == 0u16);

    // u8
    assert(2u8.pow(2u32) == 4u8);
    assert(2u8 ** 2u32 == 4u8);

    assert(2u8.pow(3u32) == 8u8);
    assert(2u8 ** 3u32 == 8u8);

    assert(4u8.pow(3u32) == 64u8);
    assert(4u8 ** 3u32 == 64u8);

    assert(3u8.pow(4u32) == 81u8);
    assert(3u8 ** 4u32 == 81u8);

    assert(10u8.pow(2u32) == 100u8);
    assert(10u8 ** 2u32 == 100u8);

    assert(5u8.pow(3u32) == 125u8);
    assert(5u8 ** 3u32 == 125u8);

    assert(3u8.pow(5u32) == 243u8);
    assert(3u8 ** 5u32 == 243u8);

    assert(2u8.pow(0u32) == 1u8);
    assert(2u8 ** 0u32 == 1u8);

    assert(0u8.pow(1u32) == 0u8);
    assert(0u8 ** 1u32 == 0u8);

    assert(0u8.pow(2u32) == 0u8);
    assert(0u8 ** 2u32 == 0u8);
}

#[test]
fn logarithmic_test_math_sw() {
    use ::assert::*;

    let max_u64 = u64::max();
    let max_u32 = u32::max();
    let max_u16 = u16::max();
    let max_u8 = u8::max();

    // u64
    assert(2.log(2) == 1);
    assert(2.log2() == 1);
    assert(1.log(3) == 0);
    assert(8.log(2) == 3);
    assert(8.log2() == 3);
    assert(100.log(10) == 2);
    assert(100.log(2) == 6);
    assert(100.log2() == 6);
    assert(100.log(9) == 2);
    assert(max_u64.log(10) == 19);
    assert(max_u64.log(2) == 63);
    assert(max_u64.log2() == 63);

    // u32
    assert(2u32.log(2u32) == 1u32);
    assert(100u32.log(10u32) == 2u32);
    assert(125u32.log(5u32) == 3u32);
    assert(256u32.log(4u32) == 4u32);
    assert(max_u32.log(10) == 9);
    assert(max_u32.log(2) == 31);
    assert(max_u32.log2() == 31);

    // u16
    assert(7u16.log(7u16) == 1u16);
    assert(49u16.log(7u16) == 2u16);
    assert(27u16.log(3u16) == 3u16);
    assert(1024u16.log(2u16) == 10u16);
    assert(max_u16.log(10) == 4);
    assert(max_u16.log(2) == 15);
    assert(max_u16.log2() == 15);

    // u8
    assert(20u8.log(20u8) == 1u8);
    assert(81u8.log(9u8) == 2u8);
    assert(36u8.log(6u8) == 2u8);
    assert(125u8.log(5u8) == 3u8);
    assert(max_u8.log(10) == 2);
    assert(max_u8.log(2) == 7);
    assert(max_u8.log2() == 7);
}
