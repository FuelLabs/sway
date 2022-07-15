script;

use std::{assert::assert, tx::tx_input_pointer};

fn main() -> bool {
    asm(r1, r2: tx_input_pointer() ) {
        gtf r1 r2 i1;
        r1: u64
    };

    true
}
