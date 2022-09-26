script;

use std::assert::assert;
use std::I64;

fn main() -> bool {
    let one = ~I64::from_uint(1u64);
    let mut res = one + ~I64::from_uint(1u64);
    assert(res == ~I64::from_uint(2u64));

    res = ~I64::from_uint(10u64) - ~I64::from_uint(11u64);
    assert(res == ~I64::from(9223372036854775807u64));

    res = ~I64::from_uint(10u64) * ~I64::neg_from(1);
    assert(res == ~I64::neg_from(10));

    res = ~I64::from_uint(10u64) * ~I64::from_uint(10u64);
    assert(res == ~I64::from_uint(100u64));

    res = ~I64::from_uint(10u64) / ~I64::from(9223372036854775807u64);
    assert(res == ~I64::neg_from(10u64));

    res = ~I64::from_uint(10u64) / ~I64::from_uint(5u64);
    assert(res == ~I64::from_uint(2u64));

    true
}
