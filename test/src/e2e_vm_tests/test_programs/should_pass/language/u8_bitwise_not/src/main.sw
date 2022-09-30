script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    let a: u8 = 2u8;

    assert(~u8::binary_not(a) == 253u8);

    true
}
