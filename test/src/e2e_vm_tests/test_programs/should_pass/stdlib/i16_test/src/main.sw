script;

use std::assert::assert;
use std::i16::*;

fn main() -> bool {
    let one = ~i16::from_uint(1u16);
    let mut res = one + ~i16::from_uint(1u16);
    assert(res == ~i16::from_uint(2u16));

    res = ~i16::from_uint(10u16) - ~i16::from_uint(11u16);
    assert(res == ~i16::from(32767u16));

    res = ~i16::from_uint(10u16) * ~i16::neg_from(1u16);
    assert(res == ~i16::neg_from(10u16));

    res = ~i16::from_uint(10u16) * ~i16::from_uint(10u16);
    assert(res == ~i16::from_uint(100u16));

    res = ~i16::from_uint(10u16) / ~i16::neg_from(1u16);
    assert(res == ~i16::neg_from(10u16));

    res = ~i16::from_uint(10u16) / ~i16::from_uint(5u16);
    assert(res == ~i16::from_uint(2u16));

    true
}
