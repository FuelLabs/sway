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

struct MyPoint<T> where T: MyAdd {
    x: T,
    y: T,
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
    0u8
}
