script;

use std::ops::{Eq, Ord};

enum X {
    Y: (),
}

#[cfg(experimental_partial_eq = false)]
impl Eq for X {
    fn eq(self, other: Self) -> bool {
        true
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for X {
    fn eq(self, other: Self) -> bool {
        true
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for X {}

impl Ord for X {
    fn lt(self, other: Self) -> bool {
        false
    }
    fn gt(self, other: Self) -> bool {
        false
    }
}

fn main() -> bool {
    X::Y == X::Y
}
