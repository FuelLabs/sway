script;

use std::assert::*;

enum Initialized {
    True: (),
    False: (),
}

#[cfg(experimental_partial_eq = false)]
impl Eq for Initialized {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Initialized::True, Initialized::True) => true,
            (Initialized::False, Initialized::False) => true,
            _ => false,
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for Initialized {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Initialized::True, Initialized::True) => true,
            (Initialized::False, Initialized::False) => true,
            _ => false,
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for Initialized {}

impl Initialized {
    fn foo(self) -> bool {
        match self {
            Self::True(_) => true,
            Self::False(_) => false,
        }
    }
}

fn main() -> u64 {
    let a = Initialized::True;
    let b = Initialized::False;
    let c = a == b;
    assert(c == false);

    assert(a.foo());

    1
}
