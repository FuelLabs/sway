script;

use std::assert::*;

fn main() {
    assert(__slice(&[1u8], 0, 1).len() == 1);
}
