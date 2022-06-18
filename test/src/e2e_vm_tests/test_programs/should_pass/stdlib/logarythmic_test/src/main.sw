script;

use std::{assert::assert, math::*};
use std::revert::revert;
use core::num::*;

fn main() -> bool {
    let max_u64 = ~u64::max();
    let max_u32 = ~u32::max();
    let max_u16 = ~u16::max();
    let max_u8 = ~u8::max();

    // u64
    assert(2.log(2) == 1);
    assert(1.log(3) == 0);
    assert(8.log(2) == 3);
    assert(100.log(10) == 2);
    assert(100.log(2) == 6);
    assert(100.log(9) == 2);

    // // u32
    // assert(2u32.log(2u32) == 4u32);
    // assert(2u32.log(3u32) == 8u32);
    // assert(42u32.log(2u32) == 1764u32);
    // assert(100u32.log(4u32) == 100000000u32);
    // assert(2u32.log(0u32) == 1u32);
    // assert(0u32.log(1u32) == 0u32);
    // assert(0u32.log(2u32) == 0u32);

    // // u16
    // assert(2u16.log(2u16) == 4u16);
    // assert(2u16.log(3u16) == 8u16);
    // assert(42u16.log(2u16) == 1764u16);
    // assert(20u16.log(3u16) == 8000u16);
    // assert(15u16.log(4u16) == 50625u16);
    // assert(2u16.log(0u16) == 1u16);
    // assert(0u16.log(1u16) == 0u16);
    // assert(0u16.log(2u16) == 0u16);

    // // u8
    // assert(2u8.log(2u8) == 4u8);
    // assert(2u8.log(3u8) == 8u8);
    // assert(4u8.log(3u8) == 64u8);
    // assert(3u8.log(4u8) == 81u8);
    // assert(10u8.log(2u8) == 100u8);
    // assert(5u8.log(3u8) == 125u8);
    // assert(3u8.log(5u8) == 243u8);
    // assert(2u8.log(0u8) == 1u8);
    // assert(0u8.log(1u8) == 0u8);
    // assert(0u8.log(2u8) == 0u8);

    true
}
