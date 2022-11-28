script;

use core::ops::*;

fn test_ok_or<T, E>(val: T, default: E) where T: Eq {
    match Option::Some(val).ok_or(default) {
        Result::Ok(inner) => assert(inner == val),
        Result::Err(_) => revert(0),
    };
}

fn main() {}
