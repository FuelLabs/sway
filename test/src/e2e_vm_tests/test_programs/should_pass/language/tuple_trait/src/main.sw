script;

use core::ops::*;

impl Eq for (u64, u64, u64) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl Eq for (u64, u64) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

fn main() -> bool {
    let t = (42, 43);
    assert(t == t);

    true
}
