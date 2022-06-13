script;

use std::assert::assert;

struct A {
    a: u64,
    b: u64,
}

impl A {
    fn f(mut self) {
        self.a = 42;
        self.b = 77;
    }

    fn g(mut self, inc: u64) {
        self.a = self.a + inc;
        self.b = self.b + inc;
    }
}

fn main() -> bool {
    let mut a = A {
        a: 0,
        b: 0,
    };
    a.f();
    assert(a.a == 42);
    assert(a.b == 77);

    a.g(1);
    assert(a.a == 43);
    assert(a.b == 78);

    true
}
