script;

use core::ops::*;

trait MyAdd {
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

struct MyPoint<T> {
    x: T,
    y: T,
}

fn add_points<T>(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> where T: MyAdd {
    MyPoint {
        x: a.x.my_add(b.x),
        y: a.y.my_add(b.y),
    }
}

fn add_points2<T, F>(a: MyPoint<T>, b: MyPoint<F>) -> MyPoint<F> where T: MyAdd, F: MyAdd {
    MyPoint {
        x: b.x.my_add(b.x),
        y: b.y.my_add(b.y),
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

fn mul_points<T>(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> where T: MyMul {
    MyPoint {
        x: a.x.my_mul(b.x),
        y: a.y.my_mul(b.y),
    }
}

fn mul_points2<T, F>(a: MyPoint<T>, b: MyPoint<F>) -> MyPoint<F> where T: MyMul, F: MyMul {
    MyPoint {
        x: b.x.my_mul(b.x),
        y: b.y.my_mul(b.y),
    }
}

fn do_math<T>(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> where T: MyAdd + MyMul {
    MyPoint {
        x: a.x.my_add(b.x),
        y: a.y.my_mul(b.y),
    }
}

fn do_math2<T, F>(a: MyPoint<T>, b: MyPoint<F>) -> MyPoint<F> where T: MyAdd + MyMul, F: MyMul + MyAdd {
    MyPoint {
        x: b.x.my_add(b.x),
        y: b.y.my_mul(b.y),
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

fn do_math3<T>(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> where T: MyMath {
    MyPoint {
        x: a.x.my_double().my_mul(b.x.my_double()),
        y: a.y.my_pow_2().my_add(b.y.my_pow_2()),
    }
}

enum MyOption<T> {
    Some: T,
    None: ()
}

impl<T> MyOption<T> {
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            MyOption::Some(v) => Result::Ok(v),
            MyOption::None => Result::Err(err),
        }
    }
}

fn test_ok_or<T, E>(val: T, default: E) where T: Eq, E: Eq {
    match MyOption::Some(val).ok_or(default) {
        Result::Ok(inner) => assert(val == inner),
        Result::Err(_) => revert(0),
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

    let c = add_points(a, b);
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
    let f = add_points2(d, e);
    assert(f.x == 10u8);
    assert(f.y == 20u8);

    let g = mul_points(a, b);
    assert(g.x == 3u64);
    assert(g.y == 8u64);

    let h = mul_points2(d, e);
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
    let k = do_math(i, j);
    assert(k.x == 12u16);
    assert(k.y == 72u16);

    let l = do_math2(i, j);
    assert(l.x == 18u16);
    assert(l.y == 144u16);

    let m = do_math3(a, b);
    assert(m.x == 12u64);
    assert(m.y == 20u64);

    // TODO(Esdrubal): reactivate this once fix in #3621 is merged
    //test_ok_or(true, 0);

    42
}
