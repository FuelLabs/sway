library u128;

use ::assert::assert;

// U128 represented as two components of a base-(2**64) number : (upper, lower) , where value = (2**64)^upper + lower
pub struct U128 {
    upper: u64,
    lower: u64,
}

pub trait From {
    fn from(h: u64, l: u64) -> Self;
} {
}

impl core::ops::Eq for U128 {
    fn eq(self, other: Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

/// Function for creating U128 from its u64 components
impl From for U128 {
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

    // Add a U128 to a U128
    fn add(self, other: U128) -> U128 {
        let lower = self.lower + other.lower;
        let mut upper = self.upper + other.upper;

        // If overflow has occurred in the lower component addition, carry
        if lower <= self.lower {
            upper = upper + 1;
        };

        // If overflow has occurred in the upper component addition, panic
        assert(upper >= self.upper);

        U128 {
            upper: upper,
            lower: lower,
        }
    }

    // Add a u64 to a U128
    fn add_u64(self, other: u64) -> U128 {
        let other_u128 = U128{upper: 0, lower: other};
        self.sub(other_u128)
    }

    // Subtract a U128 from a U128
    fn sub(self, other: U128) -> U128 {
        let mut upper = self.upper - other.upper;
        let mut lower = 0;

        // If necessary, borrow and carry for lower subtraction
        if self.lower < other.lower {
            let max = 18446744073709551615;
            let lower = max - (other.lower - self.lower - 1);
            upper = upper - 1;
        } else {
            let lower = self.lower - other.lower;
        };

        // If upper component has underflowed, panic
        assert(upper < self.upper);

        U128 {
            upper: upper,
            lower: lower,
        }
    }

    // Subtract a u64 from a U128
    fn sub_u64(self, other: u64) -> U128 {
        let other_u128 = U128{upper: 0, lower: other};
        self.sub(other_u128)
    }

    // Divide a U128 by a U64, returning a U64
    fn div_by_u64(self, other: u64) -> u64 {
        // TO DO
        42
    }s

    // TO DO : Other U128 ops : mul, div, inequalities, etc.
}

// Get lower 32 bits of a u64
fn lower(a: u64) -> u64 {
    (a << 32) >> 32
}

// Get upper 32 bits of a u64
fn upper(a: u64) -> u64 {
    a >> 32
}

// Multiply two u64 values, producing a U128
pub fn mul64(a: u64, b: u64) -> U128 {
    // Split a and b into 32-bit low and high components
    let a_lo: u64 = lower(a);
    let a_hi: u64 = upper(a);
    let b_lo: u64 = lower(b);
    let b_hi: u64 = upper(b);

    // Calculate low, high, and mid multiplications
    let ab_hi: u64 = a_hi * b_hi;
    let ab_mid: u64 = a_hi * b_lo;
    let ba_mid: u64 = b_hi * a_lo;
    let ab_lo: u64 = a_lo * b_lo;

    // low result is just the overflow from the multiplication of a and b
    let result_lo = a * b;

    // Calculate the carry
    let carry: u64 = upper(lower(ab_mid) + upper(ab_lo) + lower(ba_mid));

    // High result
    let result_hi = ab_hi + upper(ab_mid) + upper(ba_mid) + carry;

    U128 {
        upper: result_hi,
        lower: result_lo,
    }
}
