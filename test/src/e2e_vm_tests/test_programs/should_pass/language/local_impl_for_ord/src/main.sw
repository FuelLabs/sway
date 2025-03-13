script;

use std::ops::{Eq, Ord};

enum X {
    Y: (),
}

impl PartialEq for X {
    fn eq(self, other: Self) -> bool {
        true
    }
}
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
