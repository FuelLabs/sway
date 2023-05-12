script;

use core::ops::*;

fn test_ok_or<T, E>(val: T, default: E) where T: Eq {
    match Some(val).ok_or(default) {
        Ok(inner) => assert(inner == val),
        Err(_) => revert(0),
    };
}

fn main() {}
