script;

struct S {
    n: str[17],
    v: u64,
}

struct A {
    s: S,
    a: u64,
    b: bool,
}

enum B {
    B: A,
}

fn main() -> u64 {
    if let B::B(b) = B::B(A { s: S { n: " an odd length", v: 20 }, a: 10, b: false }) {
        b.a
    } else {
        0
    }
}
