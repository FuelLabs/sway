script;

trait A {
    fn a();
}

trait B: A {
    fn b();
}

struct X { x: u64 }

impl B for X {
    fn b() { }
// This code shouldn't compile because the implementation of `a()` below is missing.
//    fn a() { } 
}

fn main() { } 
