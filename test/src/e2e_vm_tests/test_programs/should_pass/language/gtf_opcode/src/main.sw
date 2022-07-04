script;

use std::assert::assert;

fn main() -> bool {
    const TX_POINTER = 42;
    asm(r1, r2: TX_POINTER ) {
        gtf r1 r2 i1;
        r1: u64
    };

    true
}
