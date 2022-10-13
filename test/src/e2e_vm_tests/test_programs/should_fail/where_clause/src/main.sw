script;

use core::ops::Add;

trait MyAdd<A> where A: Add {
    fn my_add(self, other: A) -> Self;
}

impl<A> MyAdd<A> for u64 where A: Add {
    fn my_add(self, other: A) -> u64 {
        self + other
    }
}

fn main() -> u8 {
    42
}
