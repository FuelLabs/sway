script;

use std::assert::assert;
use std::I32;

fn main() -> bool {
    let one = ~I32::from_uint(1u32);
    let mut res = one + ~I32::from_uint(1u32);
    assert(res == ~I32::from_uint(2u32));

    res = ~I32::from_uint(10u32) - ~I32::from_uint(11u32);
    assert(res == ~I32::from(2147483647u32));

    res = ~I32::from_uint(10u32) * ~I32::neg_from(1u32);
    assert(res == ~I32::neg_from(10u32));

    res = ~I32::from_uint(10u32) * ~I32::from_uint(10u32);
    assert(res == ~I32::from_uint(100u32));

    res = ~I32::from_uint(10u32) / ~I32::neg_from(1u32);
    assert(res == ~I32::neg_from(10u32));

    res = ~I32::from_uint(10u32) / ~I32::from_uint(5u32);
    assert(res == ~I32::from_uint(2u32));

    true
}
