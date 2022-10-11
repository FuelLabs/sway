script;

use core::ops::*;
use std::assert::assert;
use std::logging::log;

fn main() -> bool {
    let a: u8 = 2u8;

    let b = ~u8::binary_not(a);

    log(111111111);
    log(a);
    log(222222222);
    log(b);
    log(333333333);

    assert(~u8::binary_not(a) == 253u8);

    true
}
