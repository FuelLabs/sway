script;

use std::assert::assert;
use std::ufp128::*;

fn main() -> bool {
    // arithmetic
    let one = ~UFP128::from(1, 0);
    let two = ~UFP128::from(2, 0);
    let mut res = two + one;
    assert(~UFP128::from(3, 0) == res);

    let ufp_128_10 = ~UFP128::from(10, 0);
    res = ufp_128_10 + two;
    assert(~UFP128::from(12, 0) == res);

    let ufp_128_48 = ~UFP128::from(48, 0);
    let six = ~UFP128::from(6, 0);
    res = ufp_128_48 - six;
    assert(~UFP128::from(42, 0) == res);

    let ufp_128_169 = ~UFP128::from(169, 0);
    let ufp_128_13 = ~UFP128::from(13, 0);
    res = ufp_128_169 - ufp_128_13; 
    assert(~UFP128::from(156, 0) == res);

    // recip
    let mut value = UFP128 {
        value: 1 << 64 + 3,
    };
    res = ~UFP128::recip(value);
    assert(UFP128 {
        value: 536870912, 
    }
    == res);

    // trunc
    value = UFP128 {
        value: (1 << 64) + 3, 
    };
    res = value.trunc();
    assert(~UFP128::from_uint(1) == res);

    // floor
    value = UFP128 {
        value: (1 << 64) + 3, 
    };
    res = value.floor();
    assert(~UFP128::from_uint(1) == res);

    // fract
    value = UFP128 {
        value: (1 << 64) + 3, 
    };
    res = value.fract();
    assert(UFP128 {
        value: 3, 
    }
    == res);

    value = ~UFP128::from_uint(1);
    res = value.fract();
    assert(~UFP128::from_uint(0) == res);

    // ceil
    value = UFP128 {
            value: (1 << 64) + 3,
        };
    res = value.ceil();
    assert(~UFP128::from_uint(2) == res);

    value = ~UFP128::from_uint(1);
    res = value.ceil();
    assert(~UFP128::from_uint(1) == res);

    // round
    value = UFP128 {
            value: (1 << 64) + 3,
        };
    res = value.round();
    assert(~UFP128::from_uint(1) == res);

    value = UFP128 {
            value: (1 << 64) + (1 << 63) + 1,
        };
    res = value.round();
    assert(~UFP128::from_uint(2) == res);

    true
}
