script;

use core::num::*;
use std::assert::assert;

fn main() -> bool {
    let a: u8 = 2u32;

    assert(a.binary_not() == 4294967293u32);

    true
}
