script;

use std::assert::assert;
use std::i64::*;

fn main() -> bool {
    let one = ~i64::from_uint(1u64);
    let mut res = one + ~i64::from_uint(1u64);
    assert(res == ~i64::from_uint(2u64));

    res = ~i64::from_uint(10u64) - ~i64::from_uint(11u64);
    assert(res == ~i64::from(9223372036854775807u64));

    res = ~i64::from_uint(10u64) * ~i64::neg_from(1);
    assert(res == ~i64::neg_from(10));

    res = ~i64::from_uint(10u64) * ~i64::from_uint(10u64);
    assert(res == ~i64::from_uint(100u64));

    res = ~i64::from_uint(10u64) / ~i64::from(9223372036854775807u64);
    assert(res == ~i64::neg_from(10u64));

    res = ~i64::from_uint(10u64) / ~i64::from_uint(5u64);
    assert(res == ~i64::from_uint(2u64));

    true
}
