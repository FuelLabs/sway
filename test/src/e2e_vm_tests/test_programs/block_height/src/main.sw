script;

use std::block::height;
use std::chain::assert;

fn main() -> bool {
    let height = height();
    assert(height >= 1);
    true
}
