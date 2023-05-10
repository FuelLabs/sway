script;

use std::block::height;

fn main() -> bool {
    let h = height();
    assert(h >= 1);
    true
}
