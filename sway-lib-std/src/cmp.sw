library;

use core::ops::{Ord, Eq, OrdEq};

use ::assert::assert;

pub enum Ordering {
    Less: (),
    Equal: (),
    Greater: (),
}

pub trait Cmp {
    fn cmp(self, other: Self) -> Ordering;
} {
    /// Returns the maximum of the two values.
    fn max(self, other: Self) -> Self {
        match self.cmp(other) {
            Ordering::Less => other,
            _ => self,
        }
    }

    /// Returns the minimum of the two values.
    fn min(self, other: Self) -> Self {
        match self.cmp(other) {
            Ordering::Greater => other,
            _ => self,
        }
    }

    /// Limits the value to the range [min, max].
    fn clamp(self, min: Self, max: Self) -> Self {
        match self.cmp(min) {
            Ordering::Less => min,
            _ => match self.cmp(max) {
                Ordering::Greater => max,
                _ => self,
            },
        }
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