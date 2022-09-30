script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    let a: u64 = 2u64;

    assert(~u64::binary_not(a) == 18446744073709551613u64);

    true
}
