script;

enum S {
    A: (),
    B: bool,
}

struct S {
    a: bool,
    b: bool,
}

fn main() -> u64 {
    let s = S::A;
    let t = S { a: false, b: true };
    0
}