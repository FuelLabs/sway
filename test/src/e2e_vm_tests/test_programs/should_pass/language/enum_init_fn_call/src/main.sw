script;

enum A {
    A: u64,
}

fn main() -> u64 {
    if let A::A(n) = A::A(f()) { n } else { 0 }
}

fn f() -> u64 {
    1
}
