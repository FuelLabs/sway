script;

use std::assert::assert;

struct T {
    t1: u64, 
    t2: u64
}

enum A {
    A: u64,
    B: T,
}

fn main() -> u64 {
    let x = if let A::A(n) = A::A(f()) { n } else { 0 };
    assert(x == 1);

    let y = if let A::B(t) = A::B(g()) { t.t1 } else { 0 };
    assert(x == 42);

    1
}

fn f() -> u64 {
    1
}

fn g() -> T {
    T { t1: 42, t2: 7 }
}
