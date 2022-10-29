script;

trait MyAdd {
    fn my_add(a: Self, b: Self) -> Self;
}

struct MyU32 {
    value: u32
}

struct MyU64 {
    value: u64
}

impl MyAdd for MyU32 {
    fn my_add(a: MyU32, b: MyU32) -> MyU32 {
        MyU32 {
            value: a.value + b.value
        }
    }
}

impl MyAdd for MyU64 {
    fn my_add(a: MyU64, b: MyU64) -> MyU64 {
        MyU64 {
            value: a.value + b.value
        }
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

fn main() -> u8 {
    let foo = MyPoint {
        x: 1u32,
        y: 2u64,
    };
    let bar = MyPoint {
        x: 3u32,
        y: 4u64,
    };
    let baz = add_points(foo, bar);
    baz.y
}
