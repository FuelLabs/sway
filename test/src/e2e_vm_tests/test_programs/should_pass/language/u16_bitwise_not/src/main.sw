script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    let a: u16 = 2u16;

    assert(~u16::binary_not(a) == 65533u16);

    true
}
