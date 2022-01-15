script;

struct A {
    a: u64,
}

fn main() -> u64 {
    let a = true;
    let a = if a { 12 } else { 21 };
    let a = A { a: a };
    a.a
}
