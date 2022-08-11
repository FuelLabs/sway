library foo;

struct S {}

pub struct F {}

trait T {
    fn bar(t: S) -> S; // pub by default
}

impl T for F {
    fn bar(s: S) -> S {
        s
    }
}