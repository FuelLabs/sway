library;

pub trait MyEq2 {
    fn my_eq2(self, other: Self) -> bool;
}

impl MyEq2 for u64 {
    fn my_eq2(self, other: Self) -> bool {
        self == other
    }
}
