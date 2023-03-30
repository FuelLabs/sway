contract;

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

// Downcast from u64 to u32, losing precision
fn u64_to_u32(a: u64) -> u32 {
    let result: u32 = a;
    result
}

// Multiply two u64 values, producing a U128
pub fn mul64(a: u64, b: u64) -> U128 {
    // Split a and b into 32-bit lo and hi components
    let a_lo = u64_to_u32(a);
    let a_hi = u64_to_u32(a >> 32);
    let b_lo = u64_to_u32(b);
    let b_hi = u64_to_u32(b >> 32);

    // Calculate low, high, and mid multiplications
    let ab_hi: u64 = a_hi * b_hi;
    let ab_mid: u64 = a_hi * b_lo;
    let ba_mid: u64 = b_hi * a_lo;
    let ab_lo: u64 = a_lo * b_lo;

    // Calculate the carry bit
    let carry_bit: u64 = (u64_to_u32(ab_mid) + u64_to_u32(ba_mid) + (ab_lo >> 32)) >> 32;

    // low result is what's left after the (overflowing) multiplication of a and b
    let result_lo: u64 = a * b;

    // High result
    let result_hi: u64 = ab_hi + (ab_mid >> 32) + (ba_mid >> 32) + carry_bit;

    U128 {
        upper: result_hi,
        lower: result_lo,
    }
}
