library never;

use ::ops::{Not, Eq, Ord};

pub enum Never {}

impl Not for Never {
    fn not(self) -> Self {
        match self {}
    }
}

impl Eq for Never {
    fn eq(self, other: Self) -> bool {
        self
    }
}

impl Ord for Never {
    fn gt(self, other: Self) -> bool {
        self
    }
    fn lt(self, other: Self) -> bool {
        self
    }
}
