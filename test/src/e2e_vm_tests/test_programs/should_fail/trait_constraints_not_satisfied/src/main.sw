script;

use std::{
    assert::assert,
    logging::log,
};

trait MyAdd {
    fn my_add(self, other: Self) -> Self;
}

// this commented out code causes the trait constraints to not be satisfied

// impl MyAdd for u8 {
//     fn my_add(self, other: Self) -> Self {
//         self + other
//     }
// }

// impl MyAdd for u64 {
//     fn my_add(self, other: Self) -> Self {
//         self + other
//     }
// }

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
        x: 7u64,
        y: 9u64,
    };
    let e = MyPoint {
        x: 100u8,
        y: 10u8
    };
    let f = add_points2(d, e);
    assert(f.x == 200u8);
    assert(f.y == 20u8);

    42
}
