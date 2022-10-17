script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    assert(2u32.not() == 4294967293u32);

    true
}
