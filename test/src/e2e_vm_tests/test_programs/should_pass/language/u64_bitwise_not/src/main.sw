script;

use core::num::*;
use std::assert::assert;

fn main() -> bool {
    let a: u8 = 2u64;

    assert(a.binary_not() == 18446744073709551613u64);

    true
}
