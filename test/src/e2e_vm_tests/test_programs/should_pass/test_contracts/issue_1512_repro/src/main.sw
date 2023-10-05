contract;

use std::primitive_conversions::{u32::*, u64::*};

abi U128Contract {
    fn multiply_u64(a: u64, b: u64) -> (u64, u64);
}

impl U128Contract for Contract {
    fn multiply_u64(a: u64, b: u64) -> (u64, u64) {
        let result_u128: U128 = mul64(a, b);
        (result_u128.upper, result_u128.lower)
    }
}

// U128 represented as two components of a base-(2**64) number : (upper, lower) , where value = (2**64)^upper + lower
pub struct U128 {
    upper: u64,
    lower: u64,
}

pub trait AltFrom {
    fn from(h: u64, l: u64) -> Self;
} {
}

impl core::ops::Eq for U128 {
    fn eq(self, other: Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

/// Function for creating U128 from its u64 components
impl AltFrom for U128 {
    fn from(h: u64, l: u64) -> U128 {
        U128 {
            upper: h,
            lower: l,
        }
    }
}

/// Methods on the U128 type
impl U128 {
    /// Initializes a new, zeroed U128.
    fn new() -> U128 {
        U128 {
            upper: 0,
            lower: 0,
        }
    }

    fn add(self, other: U128) -> U128 {
        let lower = self.lower + other.lower;
        let mut upper = self.upper + other.upper;

        // If overflow has occurred in the lower component addition, carry
        if lower <= self.lower {
            upper = upper + 1;
        };

        // If overflow has occurred in the upper component addition, panic
        // assert(upper >= self.upper);

        U128 {
            upper: upper,
            lower: lower,
        }
    }

    fn sub(self, other: U128) -> U128 {
        let mut upper = self.upper - other.upper;
        let mut lower = 0;

        // If necessary, borrow and carry for lower subtraction
        if self.lower < other.lower {
            let max = 18446744073709551615;
            let _lower = max - (other.lower - self.lower - 1);
            upper = upper - 1;
        } else {
            let _lower = self.lower - other.lower;
        };

        // If upper component has underflowed, panic
        // assert(upper < self.upper);

        U128 {
            upper: upper,
            lower: lower,
        }
    }

    // TO DO : mul, div, inequalities, etc.
}

// Multiply two u64 values, producing a U128
pub fn mul64(a: u64, b: u64) -> U128 {
    // Split a and b into 32-bit lo and hi components
    let a_lo = (a & 0x00000000ffffffff).try_as_u32().unwrap();
    let a_hi = (a >> 32).try_as_u32().unwrap();
    let b_lo = (b & 0x00000000ffffffff).try_as_u32().unwrap();
    let b_hi = (b >> 32).try_as_u32().unwrap();

    // Calculate low, high, and mid multiplications
    let ab_hi = (a_hi * b_hi).as_u64();
    let ab_mid = (a_hi * b_lo).as_u64();
    let ba_mid = (b_hi * a_lo).as_u64();
    let ab_lo = (a_lo * b_lo).as_u64();

    // Calculate the carry bit
    let carry_bit = (
        (
            ab_mid.try_as_u32().unwrap() +
            ba_mid.try_as_u32().unwrap() +
            (ab_lo >> 32).try_as_u32().unwrap()
        ) >> 32
    ).as_u64();

    // low result is what's left after the (overflowing) multiplication of a and b
    let result_lo: u64 = a * b;

    // High result
    let result_hi: u64 = ab_hi + (ab_mid >> 32) + (ba_mid >> 32) + carry_bit;

    U128 {
        upper: result_hi,
        lower: result_lo,
    }
}
