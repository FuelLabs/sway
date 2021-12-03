script;

use std::block::block_height;
use std::chain::assert;

fn main() -> bool {
    let bh = block_height();
    assert(bh >= 1);
    true
}
