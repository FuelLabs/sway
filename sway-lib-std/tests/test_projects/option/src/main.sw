script;

use std::panic::panic;
use std::option::*;

fn main() {
    test_some();
    test_none();
    test_unwrap_some();
}

fn test_some() {
    let o = Option::Some(42u64);

    if (!o.is_some() || o.is_none()) {
        panic(0);
    }
}

fn test_none() {
    let o = Option::None::<()>();

    if (o.is_some() || !o.is_none()) {
        panic(0);
    }
}

fn test_unwrap_some() {
    let o = Option::Some(42u64);

    let u = o.unwrap();
    if (u != 42) {
        panic(0);
    }
}
