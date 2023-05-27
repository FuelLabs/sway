script;

trait MyAdd {
    fn my_add(self, b: Self) -> Self;
}

impl MyAdd for u32 {
    fn my_add(self, b: u32) -> u32 {
        self + b
    }
}

struct MyStruct1<T> where T: MyAdd {
    x: T,
}

fn add1<T>(s: MyStruct1<T>, v: T) -> T where T: MyAdd {
    s.x.my_add(v)
}

fn add2<T>(v1: T, v2: T) -> T where T: MyAdd  {
    let s = MyStruct1 {
        x: v1,
    };
    s.x.my_add(v2)
}

fn main() -> u8 {
    let p = MyStruct1 {
        x: 1u32,
    };

    assert(p.x.my_add(2u32) == 3u32);

    assert(add1(p,2u32) == 3u32);

    assert(add2(1u32, 2u32) == 3u32);

    0u8
}
