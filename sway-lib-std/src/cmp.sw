library;

use ::assert::assert;

pub enum Ordering {
    Less,
    Equal,
    Greater,
}

pub trait Cmp {
    // Required method
    fn cmp(self, other: Self) -> Ordering;

    // Provided methods
    fn max(self, other: Self) -> Self {
        if self.cmp(other) == Ordering::Less {
            other
        } else {
            self
        }
    }

    fn min(self, other: Self) -> Self {
        if self.cmp(other) == Ordering::Greater {
            other
        } else {
            self
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
}

impl<T> Cmp for T where T: OrdEq {
    fn cmp(self, other: Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if self < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

#[test]
fn u64() {
    assert(1.cmp(2) == Ordering::Less);
    assert(2.cmp(1) == Ordering::Greater);
    assert(1.cmp(1) == Ordering::Equal);
}