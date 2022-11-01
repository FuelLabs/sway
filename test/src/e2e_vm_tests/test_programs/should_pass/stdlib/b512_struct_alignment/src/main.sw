script;

use std::b512::B512;

fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;

    let b: B512 = B512::from(hi_bits, lo_bits);

    (b.bytes)[1] == lo_bits && (b.bytes)[0] == hi_bits
}
