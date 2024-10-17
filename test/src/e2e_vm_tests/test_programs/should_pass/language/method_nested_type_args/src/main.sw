script;

trait T1 {}
trait T2 {}

struct S<T> where T: T1 {
    x: T,
}

impl<T> S<T> where T: T1{
    fn not_just_t1(self) -> u64 where T: T2  {
        Self::also_t2(self)
    }
    fn also_t2(self) -> u64 where T: T2 {
        42
    }
}

impl T1 for u8 {}
impl T1 for u64 {}
impl T2 for u64 {}

fn main() {
    let a = S::<u64>{x: 42};
	assert_eq(a.not_just_t1(), 42);
}