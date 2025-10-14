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
    #[allow(dead_code)]
    fn my_add(a: MyU32, b: MyU32) -> MyU32 {
        MyU32 {
            value: a.value + b.value
        }
    }
}

impl MyAdd for MyU64 {
    #[allow(dead_code)]
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

// Missing where T: MyAdd
impl<T> MyAdd for MyPoint<T> {
    #[allow(dead_code)]
    fn my_add(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> {
        MyPoint {
            x: a.x.my_add(b.x),
            y: a.y.my_add(b.y),
        }
    }
}

fn main() -> u8 {
    let foo = MyPoint {
        x: 1u32,
        y: 2u32,
    };
    let bar = MyPoint {
        x: 3u32,
        y: 4u32,
    };
    let baz = foo.my_add(bar);
    baz.y
}