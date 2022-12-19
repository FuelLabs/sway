script;

use std::{
    assert::assert,
    logging::log,
};

trait MyAdd {
    fn my_add(self, other: Self) -> Self;
}

trait MyMul {
    fn my_mul(self, other: Self) -> Self;
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

struct MyPoint<T> {
    x: T,
    y: T,
}

impl<T> MyPoint<T> {
    fn add_points(self, b: MyPoint<T>) -> MyPoint<T> where T: MyAdd {
        MyPoint {
            x: self.x.my_add(b.x),
            y: self.y.my_add(b.y),
        }
    }

    fn add_points2<F>(self, b: MyPoint<F>) -> MyPoint<F> where T: MyAdd, F: MyAdd {
        MyPoint {
            x: b.x.my_add(b.x),
            y: b.y.my_add(b.y),
        }
    }
}

trait MyMul {
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

impl<T> MyPoint<T> {
    fn mul_points(self, b: MyPoint<T>) -> MyPoint<T> where T: MyMul {
        MyPoint {
            x: self.x.my_mul(b.x),
            y: self.y.my_mul(b.y),
        }
    }

    fn mul_points2<F>(self, b: MyPoint<F>) -> MyPoint<F> where T: MyMul, F: MyMul {
        MyPoint {
            x: b.x.my_mul(b.x),
            y: b.y.my_mul(b.y),
        }
    }

    fn do_math(self, b: MyPoint<T>) -> MyPoint<T> where T: MyAdd + MyMul {
        MyPoint {
            x: self.x.my_add(b.x),
            y: self.y.my_mul(b.y),
        }
    }

    fn do_math2<F>(self, b: MyPoint<F>) -> MyPoint<F> where T: MyAdd + MyMul, F: MyMul + MyAdd {
        MyPoint {
            x: b.x.my_add(b.x),
            y: b.y.my_mul(b.y),
        }
    }
}
trait MyMath: MyAdd + MyMul {

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

impl<T> MyPoint<T> {
    fn do_math3(self, b: MyPoint<T>) -> MyPoint<T> where T: MyMath {
        MyPoint {
            x: self.x.my_double().my_mul(b.x.my_double()),
            y: self.y.my_pow_2().my_add(b.y.my_pow_2()),
        }
    }
}

fn main() -> u64 {
    let a = MyPoint {
        x: 1u64,
        y: 2u64,
    };
    assert(a.x == 1u64);
    assert(a.y == 2u64);

    let b = MyPoint {
        x: 3u64,
        y: 4u64,
    };
    assert(b.x == 3u64);
    assert(b.y == 4u64);

    let c = a.add_points(b);
    assert(c.x == 4u64);
    assert(c.y == 6u64);

    let d = MyPoint {
        x: 99u16,
        y: 99u16,
    };
    let e = MyPoint {
        x: 5u8,
        y: 10u8
    };
    let f = d.add_points2(e);
    assert(f.x == 10u8);
    assert(f.y == 20u8);

    let g = a.mul_points(b);
    assert(g.x == 3u64);
    assert(g.y == 8u64);

    let h = d.mul_points2(e);
    assert(h.x == 25u8);
    assert(h.y == 100u8);

    let i = MyPoint {
        x: 3u16,
        y: 6u16,
    };
    let j = MyPoint {
        x: 9u16,
        y: 12u16,
    };
    let k = i.do_math(j);
    assert(k.x == 12u16);
    assert(k.y == 72u16);

    let l = i.do_math2(j);
    assert(l.x == 18u16);
    assert(l.y == 144u16);

    let m = a.do_math3(b);
    assert(m.x == 12u64);
    assert(m.y == 20u64);

    42
}
