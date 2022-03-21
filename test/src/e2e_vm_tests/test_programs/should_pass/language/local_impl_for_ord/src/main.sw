script;

use core::ops::{Eq, Ord};

enum X {
    Y: (),
}

impl Eq for X {
    fn eq(self, other: Self) -> bool {
        true
    }
}

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
