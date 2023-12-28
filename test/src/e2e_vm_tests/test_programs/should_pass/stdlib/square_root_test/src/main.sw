script;

use std::math::*;

fn main() -> bool {
    let max_u64 = u64::max();
    let max_u32 = u32::max();
    let max_u16 = u16::max();
    let max_u8 = u8::max();

    // u64
    assert(1.sqrt() == 1);
    assert(4.sqrt() == 2);
    assert(9.sqrt() == 3);
    assert(144.sqrt() == 12);
    assert(1024.sqrt() == 32);
    assert(10000000000000000.sqrt() == 100000000);
    assert(0.sqrt() == 0);
    assert(2.sqrt() == 1);
    assert(5.sqrt() == 2);
    assert(1000.sqrt() == 31);
    assert(max_u64.sqrt() == 4294967295);

    // u32
    assert(1u32.sqrt() == 1);
    assert(4u32.sqrt() == 2);
    assert(9u32.sqrt() == 3);
    assert(144u32.sqrt() == 12);
    assert(1024u32.sqrt() == 32);
    assert(100000000u32.sqrt() == 10000);
    assert(0u32.sqrt() == 0);
    assert(2u32.sqrt() == 1);
    assert(5u32.sqrt() == 2);
    assert(1000u32.sqrt() == 31);
    assert(max_u32.sqrt() == 65535);

    // u16
    assert(1u16.sqrt() == 1);
    assert(4u16.sqrt() == 2);
    assert(9u16.sqrt() == 3);
    assert(144u16.sqrt() == 12);
    assert(1024u16.sqrt() == 32);
    assert(50625u16.sqrt() == 225);
    assert(0u16.sqrt() == 0);
    assert(2u16.sqrt() == 1);
    assert(5u16.sqrt() == 2);
    assert(1000u16.sqrt() == 31);
    assert(max_u16.sqrt() == 255);

    // u8
    assert(1u8.sqrt() == 1);
    assert(4u8.sqrt() == 2);
    assert(9u8.sqrt() == 3);
    assert(144u8.sqrt() == 12);
    assert(0u8.sqrt() == 0);
    assert(2u8.sqrt() == 1);
    assert(5u8.sqrt() == 2);
    assert(max_u8.sqrt() == 15);

    true
}
