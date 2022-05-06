library u128;

// U128 represented as two components of a base-(2**64) number : (upper, lower) , where value = (2**64)**upper + lower
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
        if lower <= self.lower && other.lower != 0 {
            upper = upper + 1;
        };

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
            let lower = max - (other.lower - self.lower);
            upper = upper - 1;
        } else {
            let lower = self.lower - other.lower;
        };

        U128 {
            upper: upper,
            lower: lower,
        }
    }

    fn mul(self, other: U128) -> U128 {
        
        U128 {
            upper: 0,
            lower: 0,
        }
    }

    fn div(self, other: U128) -> U128{
        U128 {
            upper: 0,
            lower: 0,
        }
    }
}
