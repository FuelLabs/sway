script;

use std::{
    b512::B512,
    assert::assert,
    logging::log
};

fn main() -> u64 {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    let b = ~B512::from(hi_bits, lo_bits);
    let other_b = ~B512::new();

    let a = (b.bytes)[0] != (other_b.bytes)[0];
    log((b.bytes)[0]);
    log((other_b.bytes)[0]);
    if a {
        return 2;
    }

    let b = (b.bytes)[1] == (other_b.bytes)[1];
    if b {
        return 3;
    }

    if a && b {
        1
    } else {
        0
    }
}
