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

#[cfg(experimental_partial_eq = false)]
impl Eq for Wrapper {
    fn eq(self, other: Self) -> bool {
        self.asset == other.asset
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for Wrapper {
    fn eq(self, other: Self) -> bool {
        self.asset == other.asset
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for Wrapper {}
