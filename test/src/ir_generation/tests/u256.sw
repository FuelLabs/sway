script; 

configurable {
    SOME_U256: u256 = 0x00000000000000000000000000000000000000000000000000000001u256,
}

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

fn main(a: u256) -> u256 {
    0x00000000000000000000000000000000000000000000000000000000u256.pow(10);
    a % 0x00000000000000000000000000000000000000000000000000000001u256  
}

// ::check-ir::
// check: main

// ::check-ir-optimized::
// pass: o1
// pass: fuel
// check: main