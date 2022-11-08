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

struct MyStruct {
    x: u64,
    y: u64,
}

impl Eq for MyStruct {
    fn eq(self, other: Self) -> bool {
        std::logging::log(5);
        self.x == other.x && self.y == other.y
    }
}

fn main() -> bool {
    /* Test `bool` */
    test_is_some(true);
    test_is_none(true);
    test_unwrap(true);
    test_unwrap_or(true, false);
    test_some_ok_or(true, Error::BoolError(true));
    test_none_ok_or(true, Error::BoolError(true)); // Currently fails

    /* Test `u8` */
    test_is_some(42u8);
    test_is_none(42u8);
    test_unwrap(42u8);
    test_unwrap_or(42u8, 0u8);
    test_some_ok_or(42u8, Error::U8Error(69u8));
    test_none_ok_or(42u8, Error::U8Error(69u8)); // Currently fails

    true
}

fn test_is_some<T>(val: T) {
    assert(Option::Some(val).is_some());
    assert(!Option::None.is_some());
}

fn test_is_none<T>(val: T) {
    assert(!Option::Some(val).is_none());
    assert(Option::None.is_none());
}

fn test_unwrap<T>(val: T) where T: Eq {
    assert(Option::Some(val).unwrap() == val);
}

fn test_unwrap_or<T>(val: T, default: T) where T: Eq {
    assert(Option::Some(val).unwrap_or(default) == val);
    assert(Option::None::<T>().unwrap_or(default) == default);
}

/* Currently not able to combine the two functions below due to
 * https://github.com/FuelLabs/sway/issues/3325 */
fn test_some_ok_or<T, E>(val: T, default: E) where T: Eq {
    match Option::Some(val).ok_or(default) {
        Result::Ok(inner) => assert(val == inner),
        Result::Err(_) => revert(0),
    }
}
fn test_none_ok_or<T, E>(_val: T, default: E) where E: Eq {
    match Option::None::<T>().ok_or(default) {
        Result::Ok(_) => revert(0),
        Result::Err(e) => { 
            assert(default.eq(e));
        }
    }
}
