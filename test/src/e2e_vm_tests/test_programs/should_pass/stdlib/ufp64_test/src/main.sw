script;

use std::{assert::assert, ufp64::UFP64};

fn main() -> bool {
    // arithmetic
    let one = ~UFP64::from_uint(1);
    let two = ~UFP64::from_uint(2);
    let mut res = two + one;
    assert(~UFP64::from_uint(3) == res);

    let ufp_64_10 = ~UFP64::from_uint(10);
    res = ufp_64_10 + two;
    assert(~UFP64::from_uint(12) == res);

    let ufp_64_48 = ~UFP64::from_uint(48);
    let six = ~UFP64::from_uint(6);
    res = ufp_64_48 - six;
    assert(~UFP64::from_uint(42) == res);

    let ufp_64_169 = ~UFP64::from_uint(169);
    let ufp_64_13 = ~UFP64::from_uint(13);
    res = ufp_64_169 - ufp_64_13;
    assert(~UFP64::from_uint(156) == res);

    // recip
    let mut value = UFP64 {
        value: 1 << 32 + 3,
    };
    res = ~UFP64::recip(value);
    assert(UFP64 {
        value: 536870912, 
    }
    == res);

    // trunc
    value = UFP64 {
        value: (1 << 32) + 3, 
    };
    res = value.trunc();
    assert(~UFP64::from_uint(1) == res);

    // floor
    value = UFP64 {
        value: (1 << 32) + 3, 
    };
    res = value.floor();
    assert(~UFP64::from_uint(1) == res);

    // fract
    value = UFP64 {
        value: (1 << 32) + 3, 
    };
    res = value.fract();
    assert(UFP64 {
        value: 3, 
    }
    == res);

    value = ~UFP64::from_uint(1);
    res = value.fract();
    assert(~UFP64::from_uint(0) == res);

    // ceil
    value = UFP64 {
        value: (1 << 32) + 3, 
    };
    res = value.ceil();
    assert(~UFP64::from_uint(2) == res);

    value = ~UFP64::from_uint(1);
    res = value.ceil();
    assert(~UFP64::from_uint(1) == res);

    // round
    value = UFP64 {
        value: (1 << 32) + 3, 
    };
    res = value.round();
    assert(~UFP64::from_uint(1) == res);

    value = UFP64 {
        value: (1 << 32) + (1 << 31) + 1, 
    };
    res = value.round();
    assert(~UFP64::from_uint(2) == res);

    true
}
