script;

use std::{
    b512::B512,
    assert::assert,
    logging::log
};

fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    let b = ~B512::from(hi_bits, lo_bits);
    let other_b = ~B512::new();

    (b.bytes)[0] != (other_b.bytes)[0] && (b.bytes)[1] == (other_b.bytes)[1]
}
