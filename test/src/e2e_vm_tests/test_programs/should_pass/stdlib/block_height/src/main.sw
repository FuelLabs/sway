script;

use std::{block::height, chain::assert};

fn main() -> bool {
    let h = height();
    assert(h >= 1);
    true
}
