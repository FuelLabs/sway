script;

use std::chain::panic;
use std::option::*;

fn main() {
    test_some();
    test_none();
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
