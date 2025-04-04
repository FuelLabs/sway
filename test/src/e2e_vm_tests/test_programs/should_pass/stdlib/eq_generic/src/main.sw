script;

use std::ops::*;

fn test_ok_or_eq<T, E>(val: T, default: E) where T: Eq {
    match Some(val).ok_or(default) {
        Ok(inner) => assert(inner == val),
        Err(_) => revert(0),
    };
}

fn test_ok_or_partial_eq<T, E>(val: T, default: E) where T: PartialEq {
    match Some(val).ok_or(default) {
        Ok(inner) => assert(inner == val),
        Err(_) => revert(0),
    };
}

fn test() {
    test_ok_or_eq(0, 0u8);
    test_ok_or_partial_eq(0, 0u8);
}

fn main() {
    test();
}
