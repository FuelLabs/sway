script;

use core::ops::*;

impl<T> Option<T> {
    pub fn ok_or<E>(self, err: E) -> Result<T, E> {
        match self {
            Option::Some(v) => Result::Ok(v),
            Option::None => Result::Err(err),
        }
    }
}

fn test_ok_or<T, E>(val: T, default: E) where T: Eq {
    match Option::Some(val).ok_or(default) {
        Result::Ok(inner) => assert(inner == val),
        Result::Err(_) => revert(0),
    };
}

fn main() {}
