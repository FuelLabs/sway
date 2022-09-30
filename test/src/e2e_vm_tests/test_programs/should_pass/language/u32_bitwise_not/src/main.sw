script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    let a: u32 = 2u32;

    assert(~u32::binary_not(a) == 4294967293u32);

    true
}
