script;

use core::ops::*;

enum Error {
    BoolError: bool,
    U8Error: u8,
}

impl Eq for Error {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Error::BoolError(val1), Error::BoolError(val2)) => val2 == val1,
            (Error::U8Error(val1), Error::U8Error(val2)) => val2 == val1,
            _ => false,
        }
    }
}

fn test_none_ok_or<T, E>(_val: T, default: E) where E: Eq {
    match Option::None::<T>().ok_or(default) {
        Result::Ok(_) => revert(0),
        Result::Err(e) => assert(default == e),
    }
}

fn main() -> bool {
    test_none_ok_or(true, Error::BoolError(true));
    return true;
}
