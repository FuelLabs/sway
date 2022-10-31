script;

use std::{
    assert::assert,
    logging::log,
};

trait MyAdd {
    fn my_add(self, other: Self) -> Self;
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

fn main() -> u64 {
    let foo = MyPoint {
        x: 1u64,
        y: 2u64,
    };
    assert(foo.x == 1u64);
    assert(foo.y == 2u64);

    let bar = MyPoint {
        x: 3u64,
        y: 4u64,
    };
    assert(bar.x == 3u64);
    assert(bar.y == 4u64);

    let baz = add_points(foo, bar);
    assert(baz.x == 4u64);
    assert(baz.y == 6u64);

    42
}
