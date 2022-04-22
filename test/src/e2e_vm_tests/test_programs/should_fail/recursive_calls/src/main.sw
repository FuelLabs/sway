script;

// a -> a
fn a(n: u64) -> u64 {
    a(n)
}

// b -> c -> b
fn b(n: u64) -> u64 {
    c(n)
}

fn c(n: u64) -> u64 {
    b(n)
}

// d -> e -> f -> d
fn d(n: u64) -> u64 {
    e(n)
}

fn e(n: u64) -> u64 {
    f(n)
}

fn f(n: u64) -> u64 {
    d(n)
}

// Depends on symbols 'a' and 'b' but is not recursive.
fn g(a: u64) -> u64 {
  let b = a;
  a + b
}

// main
fn main() -> u64 {
    a(1)
}
