script;

use std::block::height;
use std::chain::assert;

fn main() -> bool {
    let h = height();
    assert(h >= 1);
    true
}
