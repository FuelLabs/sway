script;

trait MyAdd {
    fn my_add(self, b: Self) -> Self;
}

impl MyAdd for u32 {
    fn my_add(self, b: u32) -> u32 {
        self + b
    }
}

enum MyEnum<T> where T: MyAdd {
    X: T,
}

fn add1<T>(e: MyEnum<T>, v: T) -> T where T: MyAdd {
    if let MyEnum::X(x) = e {
        x.my_add(v)
    } else {
        v
    }
}

fn add2<T>(v1: T, v2: T) -> T where T: MyAdd  {
    let e = MyEnum::X(v1);
    if let MyEnum::X(x) = e {
        x.my_add(v2)
    } else {
        v1.my_add(v2)
    }
}

fn main() -> u8 {
    let e = MyEnum::X(1u32);

    if let MyEnum::X(x) = e {
        assert(x.my_add(2u32) == 3u32);
    }

    assert(add1(e,2u32) == 3u32);

    assert(add2(1u32, 2u32) == 3u32);

    0u8
}
