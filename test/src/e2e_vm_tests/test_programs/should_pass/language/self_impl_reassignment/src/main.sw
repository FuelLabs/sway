script;

use std::assert::assert;

struct A {
    a: u64,
    b: u64,
}

impl A {
    fn f(self) {
        self.a = 42;
        self.b = 77;
    }
}

fn main() -> bool {
    let a = A {
        a: 0,
        b: 0,
    };
    a.f();
    assert(a.a == 42);
    assert(a.b == 77);
    true
}
