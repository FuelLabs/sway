library my_math;

use ::my_add::*;
use ::my_mul::*;

pub trait MyMath: MyAdd + MyMul {

} {
    fn my_double(self) -> Self {
        self.my_add(self)
    }

    fn my_pow_2(self) -> Self {
        self.my_mul(self)
    }
}

impl MyMath for u8 {}

impl MyMath for u16 {}

impl MyMath for u32 {}

impl MyMath for u64 {}
