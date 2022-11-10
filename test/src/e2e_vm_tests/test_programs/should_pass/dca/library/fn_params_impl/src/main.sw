library foo;

struct S {}

pub struct F {}

impl F {
    pub fn free_fn(s: S) -> S {
        s
    }
}
