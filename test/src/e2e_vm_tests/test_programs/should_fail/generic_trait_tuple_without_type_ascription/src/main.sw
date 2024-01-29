script;

pub trait MyFrom<T> {
    fn from(b: T) -> Self;
}


pub trait MyInto<T> {
    fn my_into(self) -> T;
}


impl<T, U> MyInto<U> for T
where
    U: MyFrom<T>,
{
    fn my_into(self) -> U {
        U::from(self)
    }
}

impl MyFrom<u256> for (u64, u64, u64, u64) {
    fn from(_val: u256) -> (u64, u64, u64, u64) {
        (42, 0, 0, 0)
    }
}

fn main() -> bool {
    let (_a, _b, _c, _d) = u256::min().my_into();

    true
}