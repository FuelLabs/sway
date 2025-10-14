library;

pub trait MyFrom<T> {
    fn from(b: T) -> Self;
}


pub trait MyInto<T> {
    fn into(self) -> T;
}


impl<T, U> MyInto<U> for T
where
    U: MyFrom<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}

impl MyFrom<u256> for (u64, u64, u64, u64) {
    fn from(val: u256) -> (u64, u64, u64, u64) {
        (42, 0, 0, 0)
    }
}

impl MyFrom<u256> for (u64, u64, u64, u32) {
    fn from(val: u256) -> (u64, u64, u64, u32) {
        (42, 0, 0, 0)
    }
}

pub fn main() -> bool {
    let (a, _b, _c, _d) = u256::min().into();
    ping_to_assign_type(a, 42)
}

fn ping_to_assign_type<T>(_a: T, _b: T) -> bool {
    true
}