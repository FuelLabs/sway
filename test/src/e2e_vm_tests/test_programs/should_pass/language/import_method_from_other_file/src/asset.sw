library;

use core::ops::Eq;

pub struct Asset {
    pub value: u64
}

impl Eq for Asset {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}
