script;

use core::num::*;
use std::assert::assert;

fn main() -> bool {
    let a: u8 = 2u8;

    assert(a.binary_not() == 253u8);

    true
}
