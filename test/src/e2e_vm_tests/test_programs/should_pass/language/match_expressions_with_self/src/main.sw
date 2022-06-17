script;

use core::ops::Eq;
use std::assert::*;

enum Initialized {
    True: (),
    False: (),
}

impl Eq for Initialized {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Initialized::True, Initialized::True) => true,
            (Initialized::False, Initialized::False) => true,
            _ => false,
        }
    }
}

fn main() -> u64 {
    let a = Initialized::True;
    let b = Initialized::False;
    let c = a == b;
    assert(c == false);

    1
}
