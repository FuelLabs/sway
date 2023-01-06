library my_point;

use ::my_add::*;
use ::my_mul::*;
use ::my_math::*;

pub struct MyPoint<T> {
    x: T,
    y: T,
}

impl<T> MyAdd for MyPoint<T> where T: MyAdd {
    fn my_add(self, other: Self) -> Self {
        MyPoint {
            x: self.x.my_add(other.x),
            y: self.y.my_add(other.y),
        }
    }
}

impl<T> MyMul for MyPoint<T> where T: MyMul {
    fn my_mul(self, other: Self) -> Self {
        MyPoint {
            x: self.x.my_mul(other.x),
            y: self.y.my_mul(other.y),
        }
    }
}

impl<T> MyMath for MyPoint<T> { }
