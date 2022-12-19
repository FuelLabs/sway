library traits;

dep nested_traits;

use nested_traits::*;

pub trait MyEq {
    fn my_eq(self, other: Self) -> bool;
}

impl MyEq for u64 {
    fn my_eq(self, other: Self) -> bool {
        self == other
    }
}
