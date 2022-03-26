script;

use std::chain::panic;
use std::result::*;

fn main() {
    test_ok();
    test_err();
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
