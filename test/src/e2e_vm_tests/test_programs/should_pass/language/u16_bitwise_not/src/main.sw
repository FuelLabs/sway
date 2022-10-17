script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    assert(2u16.not() == 65533u16);

    true
}
