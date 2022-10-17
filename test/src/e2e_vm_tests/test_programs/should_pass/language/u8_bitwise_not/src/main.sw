script;

use core::ops::*;
use std::assert::assert;
use std::logging::log;

fn main() -> bool {
    assert(2u8.not() == 253u8);

    true
}
