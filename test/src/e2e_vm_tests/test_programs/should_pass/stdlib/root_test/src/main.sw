script;

use std::assert::assert;
use std::u128::*;

fn main() -> bool {

    // u64
    assert(1.sqrt() == 1);
    assert(4.sqrt() == 2);
    assert(100.sqrt() == 10);
    assert(144.sqrt() == 12);
    assert(169.sqrt() == 13);

    // u32
    assert(2u32.sqrt() == 1u32);
    assert(100u32.sqrt() == 10u32);
    assert(625u32.sqrt() == 25u32);
    assert(256u32.sqrt() == 16u32);

    // u16
    assert(1u16.sqrt() == 1u16);
    assert(49u16.sqrt() == 7u16);
    assert(9u16.sqrt() == 3u16);
    assert(1024u16.sqrt() == 32u16);

    // u8
    assert(121u8.sqrt() == 11u8);
    assert(81u8.sqrt() == 9u8);
    assert(36u8.sqrt() == 6u8);
    assert(1u8.sqrt() == 1u8);

    true
}
