script;

use std::chain::*;
use std::revert::revert;

enum Result<T, E> {
    Ok: T,
    Err: E,
}

fn local_panic() {
    asm(r1: 42) {
        rvrt r1;
    }
}

fn main() -> u64 {
    // all of these should be okay, since
    // the branches that would have type errors abort control flow.
    let x = if true {
        42u64
    } else {
        revert(0)
    };
    let x: u64 = local_panic();
    let x = if let Result::Ok(ok) = Result::Ok::<u64, u64>(5) {
        ok
    } else {
        local_panic()
    };
    let x = if true {
       true 
    } else {
        return 10;
    };
    return 42;
}
