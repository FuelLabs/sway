library ufp64;
//! A wrapper around u64 type for a library for Sway for mathematical functions operating with signed 32.32-bit fixed point numbers. 

use core::num::*;
use ::assert::assert;
use ::math::*;
use ::logging::*;
use ::revert::revert;
use ::u128::U128;

pub struct UFP64 {
    value: u64 
}

impl UFP64 {
    pub fn denominator() -> u64 {
        1 << 32
    }

    pub fn zero() -> UFP64 {
        UFP64 {
            value: 0,
        }
    }

    /// The smallest value that can be represented by this type.
    pub fn min() -> UFP64 {
        UFP64 {
            value: ~u64::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> UFP64 {
        UFP64 {
            value: ~u64::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        64
    }
}

impl core::ops::Eq for UFP64 {
    pub fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}

impl core::ops::Ord for UFP64 {
    pub fn gt(self, other: Self) -> bool {
        self.value > other.value
    }

    pub fn lt(self, other: Self) -> bool {
        self.value < other.value
    }
}

impl core::ops::Add for UFP64 {
    /// Add a UFP64 to a UFP64. Panics on overflow.
    pub fn add(self, other: Self) -> Self {
        UFP64 {
            value: self.value + other.value
        }
    }
}

impl core::ops::Subtract for UFP64 {
    /// Subtract a UFP64 from a UFP64. Panics of overflow.
    pub fn subtract(self, other: Self) -> Self {
        // If trying to subtract a larger number, panic.
        assert(!(self.value < other.value));

        UFP64 {
            value: self.value - other.value
        }
    }
}

impl core::ops::Multiply for UFP64 {
    /// Multiply a UFP64 with a UFP64. Panics of overflow.
    pub fn multiply(self, other: Self) -> Self {

        let s_value = ~U128::from(0, self.value);
        let o_value = ~U128::from(0, other.value);

        let s_mul_o = s_value * o_value;
        let res_u128 = s_mul_o >> 32;
        if res_u128.upper != 0 {
            // panic on overflow
            revert(0);
        }

        UFP64 {
            value: res_u128.lower
        }
    }
}

impl core::ops::Divide for UFP64 {
    /// Divide a UFP64 by a UFP64. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
    let zero = ~UFP64::zero();
        assert(divisor != zero);

        let denominator = ~U128::from(0, ~Self::denominator());
        let self_u128 = ~U128::from(0, self.value);
        let divisor_u128 = ~U128::from(0, divisor.value);

        let res_u128 = self_u128 * denominator / divisor_u128;
        if res_u128.upper != 0 {
            // panic on overflow
            revert(0);
        }
        UFP64 {
            value: res_u128.lower
        }
    }
}



impl UFP64 {

    pub fn from_uint(uint: u64) -> UFP64 {
        UFP64 {
            value: ~Self::denominator() * uint,
        }
    }
}

impl UFP64 {

    pub fn recip(number: UFP64) -> Self {
        let one = ~UFP64::from_uint(1);
        
        let res = one / number;
        res
    }
}

impl UFP64 {

    pub fn trunc(self) -> Self {
        UFP64 {
            value: (self.value >> 32) << 32
        }
    }
}

impl UFP64 {

    pub fn floor(self) -> Self {
        return self.trunc();
    }

    pub fn fract(self) -> Self {
        UFP64 {
            value: (self.value << 32) >> 32
        }
    }
}

impl UFP64 {

    pub fn ceil(self) -> Self {
        if self.fract().value != 0 {
            let res = self.trunc() + ~UFP64::from_uint(1);
            return res;
        }
        return self;
    }
}

impl UFP64 {

    pub fn round(self) -> Self {
        let floor = self.floor();
        let ceil = self.ceil();
        let diff_self_floor = self - floor;
        let diff_ceil_self = ceil - self;
        if diff_self_floor < diff_ceil_self {
            return floor;
        } else {
            return ceil;
        }
    }  
}

impl Root for UFP64 {
    fn sqrt(self) -> Self {
        let nominator_root = self.value.sqrt();
        let nominator = nominator_root << 16;
        UFP64 {
            value: nominator
        }
    }
}

impl Exponentiate for UFP64 {
    pub fn pow(self, exponent: Self) -> Self {
        let denominator_power = 32;
        let exponent_int = exponent.value >> denominator_power;
        let nominator_pow = ~U128::from(0, self.value).pow(~U128::from(0, exponent_int));
        let nominator = nominator_pow >> denominator_power*(exponent_int - 1);
        if nominator.upper != 0 {
            // panic on overflow
            revert(0);
        }
        UFP64 {
            value: nominator.lower
        }
    }
}

// TODO: uncomment and change accordingly, when signed integers will be added
// impl Logarithm for UFP64 {
//     fn log(self, base: Self) -> Self {
//         let nominator_log = self.value.log(base);
//         let res = (nominator_log - ~u64::from(0, 64 * 2.log(base))) * ~u64::from(1, 0);
//         UFP64 {
//             value: res
//         }
//     }
// }

impl Exponent for UFP64 {
    pub fn exp(exponent: Self) -> Self {
        let one = ~UFP64::from_uint(1);
        let p2 = one / ~UFP64::from_uint(2);
        let p3 = one / ~UFP64::from_uint(6);
        let p4 = one / ~UFP64::from_uint(24);
        let p5 = one / ~UFP64::from_uint(120);
        let p6 = one / ~UFP64::from_uint(720);
        let p7 = one / ~UFP64::from_uint(5040);

        // log(p2);

        // common technique to counter loosing sugnifucant numbers in usual approximation
        let res_minus_1 = exponent + exponent * exponent * (p2 + exponent * (p3 + exponent * (p4 + exponent * (p5 + exponent * (p6 + exponent * p7)))));
        let res = res_minus_1 + one;
        res
    }
}
