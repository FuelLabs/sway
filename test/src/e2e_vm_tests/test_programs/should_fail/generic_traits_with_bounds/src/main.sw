script;

use std::assert::*;

trait Trait {
    fn method(self) -> u64;
}

impl<A, B> Trait for (A, B) where A: Trait, B: Trait {
    fn method(self) -> u64 {
        self.0.method() + self.1.method()
    }
}

/* Without this (1, 2).method() should not be found
impl Trait for u64 {
    fn method(self) -> u64 {
        self
    }
}
*/

fn main() -> bool {
    assert((1,2).method() == 3);
    true
}
