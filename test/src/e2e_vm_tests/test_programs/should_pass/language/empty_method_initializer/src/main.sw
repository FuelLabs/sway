script;

use std::b512::B512;

fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

    let b = B512::from((hi_bits, lo_bits));
    let other_b = B512::new();
    ((b.bits())[0] != (other_b.bits())[0]) && ((b.bits())[1] == (other_b.bits())[1])
}
