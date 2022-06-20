script;

use std::result::Result;
use std::revert::revert;

fn main() -> bool {
    test_ok();
    test_err();
    test_unwrap_ok();

    true
}

fn test_ok() {
    let r = Result::Ok::<u64, ()>(42u64);

    if (!r.is_ok() || r.is_err()) {
        revert(0);
    }
}

fn test_err() {
    let r = Result::Err::<(), ()>(());

    if (r.is_ok() || !r.is_err()) {
        revert(0);
    }
}

fn test_unwrap_ok() {
    let r = Result::Ok::<u64, ()>(42);

    let u = r.unwrap();
    if (u != 42) {
        revert(0);
    }
}
