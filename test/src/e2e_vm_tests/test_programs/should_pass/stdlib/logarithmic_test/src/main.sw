script;

use std::math::*;

fn main() -> bool {
    let max_u64 = ~u64::max();
    let max_u32 = ~u32::max();
    let max_u16 = ~u16::max();
    let max_u8 = ~u8::max();

    // u64
    assert(2.log(2) == 1);
    assert(2.log2() == 1);
    assert(1.log(3) == 0);
    assert(8.log(2) == 3);
    assert(8.log2() == 3);
    assert(100.log(10) == 2);
    assert(100.log(2) == 6);
    assert(100.log2 == 6);
    assert(100.log(9) == 2);
    assert(max_u64.log(10) == 19);
    assert(max_u64.log(2) == 64);
    assert(max_u64.log2() == 64);

    // u32
    assert(2u32.log(2u32) == 1u32);
    assert(100u32.log(10u32) == 2u32);
    assert(125u32.log(5u32) == 3u32);
    assert(256u32.log(4u32) == 4u32);
    assert(max_u32.log(10) == 9);
    assert(max_u32.log(2) == 31);
    assert(max_u32.log2() == 31);

    // u16
    assert(7u16.log(7u16) == 1u16);
    assert(49u16.log(7u16) == 2u16);
    assert(27u16.log(3u16) == 3u16);
    assert(1024u16.log(2u16) == 10u16);
    assert(max_u16.log(10) == 4);
    assert(max_u16.log(2) == 15);

    // u8
    assert(20u8.log(20u8) == 1u8);
    assert(81u8.log(9u8) == 2u8);
    assert(36u8.log(6u8) == 2u8);
    assert(125u8.log(5u8) == 3u8);
    assert(max_u8.log(10) == 2);
    assert(max_u8.log(2) == 7);
    
    true
}
