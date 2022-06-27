library fixed_point;
//! A wrapper around U128 type for a library for Sway for mathematical functions operating with signed 64.64-bit fixed point numbers. 

use ::u128::U128;
use ::assert::assert;
use ::math::*;
use ::logging::*;

pub struct UFP64 {
    value: U128 
}

pub trait From {
    pub fn from(int_part: u64, fract_part: u64) -> UFP64; 
}

impl From for UFP64 {
    pub fn from(int_part: u64, fract_part: u64) -> UFP64 {
        UFP64{
            value: ~U128::from(int_part, fract_part)
        } 
    }
}

impl UFP64 {
    pub fn zero() -> UFP64 {
        UFP64 {
            value: ~U128::from(0, 0),
        }
    }

    /// The smallest value that can be represented by this type.
    pub fn min() -> UFP64 {
        UFP64 {
            value: ~U128::min(),
        }
    }

    /// The largest value that can be represented by this type,
    pub fn max() -> UFP64 {
        UFP64 {
            value: ~U128::max(),
        }
    }

    /// The size of this type in bits.
    pub fn bits() -> u32 {
        128
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

        let base = ~U128::from(1,0);

        let self_up = ~U128::from(0, self.value.upper);
        let self_lo = ~U128::from(0, self.value.lower);

        let other_up = ~U128::from(0, other.value.upper);
        let other_lo = ~U128::from(0, other.value.lower);

        let mut up_up = self_up * other_up;
        up_up *= base;
        let mut lo_lo = self_lo * other_lo;
        lo_lo /= base;

        let up_lo = self_up * other_lo;
        let lo_up = self_lo * other_up;

        UFP64 {
            value: up_lo + lo_up
        }
    }
}

impl core::ops::Divide for UFP64 {
    /// Divide a UFP64 by a UFP64. Panics if divisor is zero.
    pub fn divide(self, divisor: Self) -> Self {
        let mut s = self;
        let zero = ~UFP64::zero();
        let u128_max = ~U128::max();

        let denominator = ~U128::from(1, 0);

        assert(divisor != zero);

        let mut res = ~UFP64::from(0, 0);

        if self.value.upper == 0 {
            s.value *= denominator;
            let result = s.value / divisor.value;
            res = UFP64 {
                value: result
            }
        } else {

            let inter = u128_max / divisor.value;

            if inter.upper == 0 {


                let l1 = self.value * inter;
                log("l1");
                log(l1.upper);
                log(l1.lower);

                let result = (self.value * inter) / denominator;
                res = UFP64 {
                    value: result
                }
            } else {
                let mid = ~U128::from(0, 2 << 32);

                let s = self.value / mid;

                let inter = inter / mid;

                let result = s * inter;
                res = UFP64 {
                    value: result
                }
            }
        }
        res
    }
}

impl UFP64 {

    pub fn recip(number: UFP64) -> Self {
        let one = ~U128::from(0,1);
        
        UFP64 {
            value: one / number.value
        }
    }

    pub fn floor(self) -> Self {
        ~Self::from(self.value.upper, 0)
    }

    pub fn ceil(self) -> Self {
        let val = self.value;
        if val.lower == 0 {
            return ~Self::from(val.upper, 0);
        } else {
            return ~Self::from(val.upper + 1, 0);
        }
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

    pub fn trunc(self) -> Self {
        ~Self::from(self.value.upper, 0)
    }

    pub fn fract(self) -> Self {
        ~Self::from(0, self.value.lower)
    }    
}

impl Root for UFP64 {
    fn sqrt(self) -> Self {
        let nominator_root = self.value.sqrt();
        let nominator = nominator_root * ~U128::from(0, 2.pow(32));
        ~Self::from(nominator.upper, nominator.lower)
    }
}

impl Exponentiate for UFP64 {
    pub fn pow(self, exponent: Self) -> Self {
        let nominator_pow = self.value.pow(exponent.value);
        let one_u128 = ~U128::from(0, 1);
        let two_u128 = ~U128::from(0, 2);
        let u128_64 = ~U128::from(0, 64);
        let two_pow_64_n_minus_1 = two_u128.pow(u128_64*(exponent.value - one_u128));
        let nominator = nominator_pow / two_pow_64_n_minus_1;
        ~Self::from(nominator.upper, nominator.lower)
    }
}

// TODO: uncomment and change accordingly, when signed integers will be added
// impl Logarithm for UFP64 {
//     fn log(self, base: Self) -> Self {
//         let nominator_log = self.value.log(base);
//         let res = (nominator_log - ~U128::from(0, 64 * 2.log(base))) * ~U128::from(1, 0);
//         UFP64 {
//             value: res
//         }
//     }
// }

impl Exponent for UFP64 {
    pub fn exp(exponent: Self) -> Self {
        let one = ~UFP64::from(1, 0);
        let p2 = one / ~UFP64::from(2, 0);
        let p3 = one / ~UFP64::from(6, 0);
        let p4 = one / ~UFP64::from(24, 0);
        let p5 = one / ~UFP64::from(120, 0);
        let p6 = one / ~UFP64::from(720, 0);
        let p7 = one / ~UFP64::from(5040, 0);

        log(p2.value.upper);
        log(p2.value.lower);
        log(p3.value.upper);
        log(p3.value.lower);
        log(p4.value.upper);
        log(p4.value.lower);
        log(p5.value.upper);
        log(p5.value.lower);
        log(p6.value.upper);
        log(p6.value.lower);
        log(p7.value.upper);
        log(p7.value.lower);

        // common technique to counter loosing sugnifucant numbers in usual approximation
        // let res_minus_1 = exponent + exponent * exponent * (p2 + exponent * (p3 + exponent * (p4 + exponent * (p5 + exponent * (p6 + exponent * p7)))));
        // let res = res_minus_1 + one;
        let res = one;
        res
    }
}
