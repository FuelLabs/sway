script;

// dep my_add;
// dep my_mul;
// dep my_math;
// dep my_point;
// dep uint_tests;
// dep point_tests;

use std::{
    assert::assert,
    logging::log,
};

// use my_add::*;
// use my_point::*;
// use uint_tests::*;
// use point_tests::*;

pub trait MyAdd {
    fn my_add(self, other: Self) -> Self;
}

impl MyAdd for u8 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

impl MyAdd for u16 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

impl MyAdd for u32 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

impl MyAdd for u64 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

pub trait MyMul {
    fn my_mul(self, other: Self) -> Self;
}

impl MyMul for u8 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

impl MyMul for u16 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

impl MyMul for u32 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

impl MyMul for u64 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

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

pub struct MyPoint<T> {
    x: T,
    y: T,
}

impl<T> MyPoint<T> {
    pub fn new(x: T, y: T) -> MyPoint<T> {
        MyPoint {
            x,
            y,
        }
    }
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

impl<T> MyMath for MyPoint<T> where T: MyMath { }

pub fn basic_unit_tests() {
    assert(100.my_add(99) == 199);
    assert(3.my_mul(4) == 12);
    assert(5.my_double() == 10);
    assert(5.my_pow_2() == 25);
}

pub fn basic_point_tests() {
    let a = MyPoint::new(1, 2);
    let b = MyPoint::new(3, 4);

    let c = a.my_add(b);
    assert(c.x == 4);
    assert(c.y == 6);
}

fn main() -> u64 {
    basic_unit_tests();
    basic_point_tests();

    42
}
