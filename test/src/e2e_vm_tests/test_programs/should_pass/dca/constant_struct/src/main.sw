script;

struct A {
   a : u64
}

fn foo(a : u64) -> A {
  A { a }
}

const B: A = foo(32);

fn main() -> u64 {
    B.a
}
