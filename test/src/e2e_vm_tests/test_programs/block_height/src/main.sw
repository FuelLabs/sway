script;

use std::block::block_height;

fn main() bool {
    let bh = block_height();
    assert(bh > 0);
    true
}
