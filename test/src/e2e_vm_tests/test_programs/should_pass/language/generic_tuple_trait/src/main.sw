script;

use std::assert::*;

trait Trait {
    fn method(self) -> u64;
}

trait Trait2 {
    fn method2(self) -> u64;
}

impl<A, B> Trait for (A, B) {
    fn method(self) -> u64 {
        42
    }
}

impl Trait2 for u64 {
    fn method2(self) -> u64 {
        self
    }
}

struct S {}

impl Trait2 for S {
    fn method2(self) -> u64 {
        2
    }
}

impl<A, B> Trait2 for (A, B) where A: Trait2, B: Trait2 {
    fn method2(self) -> u64 {
        self.0.method2() + self.1.method2()
    }
}

fn main() -> bool {
    assert((1,2).method() == 42);
    assert((1,2).method2() == 3);
    assert((1, S{}).method2() == 3);
    assert(((1,2),(1,2)).method2() == 6);
    assert(((1, S{}),(1,(1,2))).method2() == 7);

    true
}
