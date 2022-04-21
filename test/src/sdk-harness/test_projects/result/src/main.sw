script;

use std::panic::panic;
use std::result::*;

fn main() {
    test_ok();
    test_err();
    test_unwrap_ok();
}

fn test_ok() {
    let r = Result::Ok::<u64, ()>(42u64);

    if (!r.is_ok() || r.is_err()) {
        panic(0);
    }
}

fn test_err() {
    let r = Result::Err::<(), ()>(());

    if (r.is_ok() || !r.is_err()) {
        panic(0);
    }
}

fn test_unwrap_ok() {
    let r = Result::Ok::<u64, ()>(42);

    let u = r.unwrap();
    if (u != 42) {
        panic(0);
    }
}
