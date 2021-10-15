script;

use std::ops::Ord;

enum X {
    Y: (),
}

impl Ord for X {
    fn eq(self, other: Self) -> bool {
        true
    }
    fn lt(self, other: Self) -> bool {
        false
    }
    fn gt(self, other: Self) -> bool {
        false
    }
}

impl X {
    fn gte(self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }
    fn not_gte(self, other: Self) -> bool {
        !self.gte(other)
    }
}

fn main() -> bool {
    (X::Y).gte(X::Y)
}
