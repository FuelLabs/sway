script;

use core::num::*;
use std::assert::assert;

fn main() -> bool {
    let a: u8 = 2u16;

    assert(a.binary_not() == 65533u16);

    true
}
