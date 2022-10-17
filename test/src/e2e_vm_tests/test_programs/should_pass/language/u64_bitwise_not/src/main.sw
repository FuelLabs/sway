script;

use core::ops::*;
use std::assert::assert;

fn main() -> bool {
    assert(2u64.not() == 18446744073709551613u64);

    true
}
