script;

use std::assert::assert;
use std::i128::I128;
use std::u128::U128;

fn main() -> bool {
    let u128_one = U128 {
        upper: 0,
        lower: 1,
    };
    let u128_two = U128 {
        upper: 0,
        lower: 2,
    };
    let one = ~I128::from_uint(u128_one);
    let mut res = one + ~I128::from_uint(u128_one);
    assert(res == ~I128::from_uint(u128_two));

    let u128_10 = U128 {
        upper: 0,
        lower: 10,
    };
    let u128_11 = U128 {
        upper: 0,
        lower: 11,
    };
    res = ~I128::from_uint(u128_10) - ~I128::from_uint(u128_11);
    assert(res.underlying.lower == ~u64::max());

    res = ~I128::from_uint(u128_10) * ~I128::neg_from(u128_one);
    assert(res == ~I128::neg_from(u128_10));

    res = ~I128::from_uint(u128_10) * ~I128::from_uint(u128_10);
    let u128_100 = U128 {
        upper: 0,
        lower: 100,
    };
    assert(res == ~I128::from_uint(u128_100));

    let u128_lower_max_u64 = U128 {
        upper: 0,
        lower: ~u64::max(),
    };

    res = ~I128::from_uint(u128_10) / ~I128::from(u128_lower_max_u64);
    assert(res == ~I128::neg_from(u128_10));

    let u128_5 = U128 {
        upper: 0,
        lower: 5,
    };

    let u128_2 = U128 {
        upper: 0,
        lower: 2,
    };

    res = ~I128::from_uint(u128_10) / ~I128::from_uint(u128_5);
    assert(res == ~I128::from_uint(u128_2));

    true
}
