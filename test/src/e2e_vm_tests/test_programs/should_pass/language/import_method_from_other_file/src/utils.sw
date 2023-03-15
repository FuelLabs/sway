library;

use core::ops::Eq;
use ::asset::Asset;

pub struct Wrapper {
    asset: Asset
}

impl Wrapper {
    pub fn new(value: u64) -> Self {
        Wrapper {
            asset: Asset {
                value
            }
        }
    }
}

impl Eq for Wrapper {
    fn eq(self, other: Self) -> bool {
        self.asset == other.asset
    }
}
