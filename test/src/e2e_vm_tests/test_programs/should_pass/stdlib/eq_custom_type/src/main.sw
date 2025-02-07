script;

use core::ops::*;

enum Error {
    BoolError: bool,
    U8Error: u8,
}

#[cfg(experimental_partial_eq = false)]
impl Eq for Error {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Error::BoolError(val1), Error::BoolError(val2)) => val2 == val1,
            (Error::U8Error(val1), Error::U8Error(val2)) => val2 == val1,
            _ => false,
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for Error {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Error::BoolError(val1), Error::BoolError(val2)) => val2 == val1,
            (Error::U8Error(val1), Error::U8Error(val2)) => val2 == val1,
            _ => false,
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for Error {}

fn test_none_ok_or_eq<T, E>(_val: T, default: E)
where
    E: Eq,
{
    match None::<T>.ok_or(default) {
        Ok(_) => revert(0),
        Err(e) => assert(default == e),
    }
}

#[cfg(experimental_partial_eq = true)]
fn test_none_ok_or_partial_eq<T, E>(_val: T, default: E)
where
    E: Eq,
{
    match None::<T>.ok_or(default) {
        Ok(_) => revert(0),
        Err(e) => assert(default == e),
    }
}

#[cfg(experimental_partial_eq = false)]
fn test() {
    test_none_ok_or_eq(true, Error::BoolError(true));
}

#[cfg(experimental_partial_eq = true)]
fn test() {
    test_none_ok_or_eq(true, Error::BoolError(true));
    test_none_ok_or_partial_eq(true, Error::BoolError(true));
}

fn main() -> bool {
    test();
    return true;
}
