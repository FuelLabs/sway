script;

use std::{assert::assert, math::*};
use std::revert::revert;

fn main() -> bool {
    // u64
    assert(2.pow(2) == 4);
    assert(2 ** 2 == 4);

    assert(2.pow(3) == 8);
    assert(2 ** 3 == 8);

    assert(42.pow(2) == 1764);
    assert(42 ** 2 == 1764);

    assert(42.pow(3) == 74088);
    assert(42 ** 3 == 74088);

    assert(100.pow(5) == 10000000000);
    assert(100 ** 5 == 10000000000);

    assert(100.pow(8) == 10000000000000000);
    assert(100 ** 8 == 10000000000000000);

    assert(100.pow(9) == 1000000000000000000);
    assert(100 ** 9 == 1000000000000000000);

    assert(2.pow(0) == 1);
    assert(2 ** 0 == 1);

    assert(0.pow(1) == 0);
    assert(0 ** 1 == 0);

    assert(0.pow(2) == 0);
    assert(0 ** 2 == 0);

    // u32
    assert(2u32.pow(2u32) == 4u32);
    assert(2u32 ** 2u32 == 4u32);

    assert(2u32.pow(3u32) == 8u32);
    assert(2u32 ** 3u32 == 8u32);

    assert(42u32.pow(2u32) == 1764u32);
    assert(42u32 ** 2u32 == 1764u32);

    assert(100u32.pow(4u32) == 100000000u32);
    assert(100u32 ** 4u32 == 100000000u32);

    assert(2u32.pow(0u32) == 1u32);
    assert(2u32 ** 0u32 == 1u32);

    assert(0u32.pow(1u32) == 0u32);
    assert(0u32 ** 1u32 == 0u32);

    assert(0u32.pow(2u32) == 0u32);
    assert(0u32 ** 2u32 == 0u32);

    // u16
    assert(2u16.pow(2u16) == 4u16);
    assert(2u16 ** 2u16 == 4u16);

    assert(2u16.pow(3u16) == 8u16);
    assert(2u16 ** 3u16 == 8u16);

    assert(42u16.pow(2u16) == 1764u16);
    assert(42u16 ** 2u16 == 1764u16);

    assert(20u16.pow(3u16) == 8000u16);
    assert(20u16 ** 3u16 == 8000u16);

    assert(15u16.pow(4u16) == 50625u16);
    assert(15u16 ** 4u16 == 50625u16);

    assert(2u16.pow(0u16) == 1u16);
    assert(2u16 ** 0u16 == 1u16);

    assert(0u16.pow(1u16) == 0u16);
    assert(0u16 ** 1u16 == 0u16);

    assert(0u16.pow(2u16) == 0u16);
    assert(0u16 ** 2u16 == 0u16);

    // u8
    assert(2u8.pow(2u8) == 4u8);
    assert(2u8 ** 2u8 == 4u8);

    assert(2u8.pow(3u8) == 8u8);
    assert(2u8 ** 3u8 == 8u8);

    assert(4u8.pow(3u8) == 64u8);
    assert(4u8 ** 3u8 == 64u8);

    assert(3u8.pow(4u8) == 81u8);
    assert(3u8 ** 4u8 == 81u8);

    assert(10u8.pow(2u8) == 100u8);
    assert(10u8 ** 2u8 == 100u8);

    assert(5u8.pow(3u8) == 125u8);
    assert(5u8 ** 3u8 == 125u8);

    assert(3u8.pow(5u8) == 243u8);
    assert(3u8 ** 5u8 == 243u8);

    assert(2u8.pow(0u8) == 1u8);
    assert(2u8 ** 0u8 == 1u8);

    assert(0u8.pow(1u8) == 0u8);
    assert(0u8 ** 1u8 == 0u8);

    assert(0u8.pow(2u8) == 0u8);
    assert(0u8 ** 2u8 == 0u8);

    true
}
