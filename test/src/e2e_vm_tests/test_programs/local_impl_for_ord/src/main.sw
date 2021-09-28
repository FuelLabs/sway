script;

use std::ops::Ord;

enum X {
    Y: ()
}

impl Ord for X {
    fn eq(self, other: Self) -> bool { true }
    fn lt(self, other: Self) -> bool { false }
    fn gt(self, other: Self) -> bool { false }
}

fn main() -> bool {
    X::Y == X::Y
}
