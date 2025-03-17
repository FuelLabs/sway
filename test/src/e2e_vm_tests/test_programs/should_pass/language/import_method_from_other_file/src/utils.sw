library;

use ::asset::Asset;

pub struct Wrapper {
    pub asset: Asset,
}

impl Wrapper {
    pub fn new(value: u64) -> Self {
        Wrapper {
            asset: Asset { value },
        }
    }
}

impl PartialEq for Wrapper {
    fn eq(self, other: Self) -> bool {
        self.asset == other.asset
    }
}
impl Eq for Wrapper {}
