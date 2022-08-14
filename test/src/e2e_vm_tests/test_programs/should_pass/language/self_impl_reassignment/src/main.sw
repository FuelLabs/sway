script;

use std::{assert::assert, revert::revert};

struct A {
    a: u64,
    b: u64,
}

impl A {
    fn f(ref mut self) {
        self.a = 42;
        self.b = 77;
    }

    fn g(ref mut self, inc: u64) {
        self.a = self.a + inc;
        self.b = self.b + inc;
    }

    fn h(ref mut self) {
        self = A {
            a: 100,
            b: 200,
        }
    }
}

enum E {
    X: u64,
    Y: u64,
}

impl E {
    fn j(ref mut self, inc: u64) {
        self = match self {
            E::X(val) => E::Y(val + inc),
            E::Y(val) => E::X(val + inc),
        }
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

    a.h();
    assert(a.a == 100);
    assert(a.b == 200);

    let mut e = E::X(42);
    match e {
        E::X(42) => {},
        _ => revert(0),
    };
    
    e.j(4);
    match e {
        E::Y(46) => {},
        _ => revert(0),
    };

    e.j(5);
    match e {
        E::X(51) => {},
        _ => revert(0),
    };
   
    true
}
