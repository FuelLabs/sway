script;

use std::assert::assert;
use std::I8;

fn main() -> bool {
    let one = ~I8::from_uint(1u8);
    let mut res = one + ~I8::from_uint(1u8);
    assert(res == ~I8::from_uint(2u8));

    res = ~I8::from_uint(10u8) - ~I8::from_uint(11u8);
    assert(res == ~I8::from(127u8));

    res = ~I8::from_uint(10u8) * ~I8::from(127u8);
    assert(res == ~I8::from(118u8));

    res = ~I8::from_uint(10u8) * ~I8::from_uint(10u8);
    assert(res == ~I8::from_uint(100u8));

    res = ~I8::from_uint(10u8) / ~I8::from(127u8);
    assert(res == ~I8::from(118u8));

    res = ~I8::from_uint(10u8) / ~I8::from_uint(5u8);
    assert(res == ~I8::from_uint(2u8));

    true
}
