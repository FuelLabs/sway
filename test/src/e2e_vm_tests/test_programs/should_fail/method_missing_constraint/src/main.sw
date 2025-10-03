library;

trait T1 {}
trait T2 {}

struct S<T> where T: T1 {
    x: T,
}

impl<T> S<T> where T: T1 {
    fn only_t1(self) {
        Self::also_t2(self);
    }
    fn also_t2(self) where T: T2 { }
}

impl T1 for u8 {}
impl T1 for u64 {}
impl T2 for u64 {}

pub fn main() {
    let a = S::<u8>{ x: 42 };
    a.only_t1();
}
