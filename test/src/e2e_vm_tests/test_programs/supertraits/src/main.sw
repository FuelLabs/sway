script;

use std::chain::assert;

trait A {
    fn f(self) -> u64;
} {
    fn add_f(self, x: u64) -> u64 {
        self.f() + x
    }
    fn mul_f(self, x: u64) -> u64 {
        self.f() * x
    }
}

trait B: A {
    fn g(self) -> u64;
} {
    fn add_g(self, x: u64) -> u64 {
        self.g() + x
    }
    fn mul_g(self, x: u64) -> u64 {
        self.g() * x
    }
}

trait C: B {
    fn h(self) -> u64;
} {
    fn add_h(self, x: u64) -> u64 {
        self.h() + x
    }
    fn mul_h(self, x: u64) -> u64 {
        self.h() * x
    }
}

trait D: B + C {
    fn i(self) -> u64;
} {
    fn add_i(self, x: u64) -> u64 {
        self.i() + x
    }
    fn mul_i(self, x: u64) -> u64 {
        self.i() * x
    }
}

struct S {
    x: u64,
    y: u64,
    z: u64,
    w: u64,
}

impl A for S {
    fn f(self) -> u64 {
        self.x
    }
}

impl B for S {
    fn g(self) -> u64 {
        self.y
    }
}

impl C for S {
    fn h(self) -> u64 {
        self.z
    }
}

impl D for S {
    fn i(self) -> u64 {
        self.w
    }
}

struct U {
    x: u64,
    y: u64,
    z: u64,
    w: u64,
}

impl A for U {
    fn f(self) -> u64 {
        self.x + 1
    }
}

impl B for U {
    fn g(self) -> u64 {
        self.y + 1
    }
}

impl C for U {
    fn h(self) -> u64 {
        self.z + 1
    }
}

fn main() -> bool {
    let s = S {
        x: 1,
        y: 2,
        z: 3,
        w: 4,
    };

    assert(s.f() == 1);
    assert(s.add_f(5) == 6);
    assert(s.mul_f(5) == 5);
    assert(s.g() == 2);
    assert(s.add_g(5) == 7);
    assert(s.mul_g(5) == 10);
    assert(s.h() == 3);
    assert(s.add_h(5) == 8);
    assert(s.mul_h(5) == 15);
    assert(s.i() == 4);
    assert(s.add_i(5) == 9);
    assert(s.mul_i(5) == 20);

    let u = U {
        x: 5,
        y: 6,
        z: 7,
        w: 8,
    };

    assert(u.f() == 6);
    assert(u.add_f(5) == 11);
    assert(u.mul_f(5) == 30);
    assert(u.g() == 7);
    assert(u.add_g(5) == 12);
    assert(u.mul_g(5) == 35);
    assert(u.h() == 8);
    assert(u.add_h(5) == 13);
    assert(u.mul_h(5) == 40);

    true
}
