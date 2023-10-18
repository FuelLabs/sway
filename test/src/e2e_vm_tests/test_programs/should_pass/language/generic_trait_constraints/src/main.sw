script;

trait MyAdd<T> {
    fn my_add(self, a: T, b: T) -> T;
}

struct Struct<A> where A: MyAdd<A> {
    data: A,
}

struct Struct2<A, B> where A: MyAdd<B>, B: MyAdd<A> {
    data_a: A,
    data_b: B,
}

impl MyAdd<u64> for u64 {
    fn my_add(self, a: u64, b: u64) -> u64 {
        a + b
    }
}

fn main() -> bool {
    let s = Struct {data: 1_u64 };
    assert_eq(s.data.my_add(1,2),3);

    let s = Struct2 {data_a: 1_u64, data_b: 1_u64 };
    assert_eq(s.data_a.my_add(1,2),3);
    assert_eq(s.data_b.my_add(1,2),3);

    true
}