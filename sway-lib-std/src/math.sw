library math;

use ::revert::revert;

const EXPONENTIATION_OVERFLOW = 11;

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

pub trait Exponentiate {
    fn pow(self, exponent: Self) -> Self;
}


// NOTE shouldn't have to check `$of` when vm implements flags.
// the impl below would only be needed when setting the flag to allow overflow.
impl Exponentiate for u64 {
    fn pow(self, exponent: Self) -> Self {
        let empty_return = (0u64, 0u64);
        let(value, overflow) = asm(r1: self, r2: exponent, r3, output: empty_return, r4: 1) {
            // flag r4;
            exp r3 r1 r2;
            sw output r3 i0; // store the word at r3 in output + 0 words
            sw output of i1; // store the word at `of` in output + 1 word
            output: (u64, u64)
        };
        log_u64(overflow);
        if overflow != 0 {
            revert(EXPONENTIATION_OVERFLOW);
        };

        value
    }
}

impl Exponentiate for u32 {
    fn pow(self, exponent: Self) -> Self {
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
        }
    }
}

impl Exponentiate for u16 {
    fn pow(self, exponent: Self) -> Self {
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
        }
    }
}

impl Exponentiate for u8 {
    fn pow(self, exponent: Self) -> Self {
        asm(r1: self, r2: exponent, r3) {
            exp r3 r1 r2;
            r3: Self
        }
    }
}
