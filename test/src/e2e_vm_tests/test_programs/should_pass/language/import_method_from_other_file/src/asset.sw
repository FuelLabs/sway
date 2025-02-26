library;

use Eq;

pub struct Asset {
    pub value: u64,
}

#[cfg(experimental_partial_eq = false)]
impl Eq for Asset {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for Asset {
    fn eq(self, other: Self) -> bool {
        self.value == other.value
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for Asset {}
