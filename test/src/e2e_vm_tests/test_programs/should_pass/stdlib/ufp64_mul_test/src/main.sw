script;

use std::{assert::assert, ufp64::UFP64};

fn main() -> bool {
    let one = ~UFP64::from_uint(1);
    let two = ~UFP64::from_uint(2);
    let mut res = one * two;
    assert(two == res);

    let ufp_64_10 = ~UFP64::from_uint(10);
    let ufp_64_20 = ~UFP64::from_uint(4);
    res = ufp_64_10 * ufp_64_20;
    assert(~UFP64::from_uint(40) == res);

    let ufp_64_11 = ~UFP64::from_uint(11);
    let ufp_64_12 = ~UFP64::from_uint(12);
    res = ufp_64_11 * ufp_64_12;
    assert(~UFP64::from_uint(132) == res);

    let ufp_64_150 = ~UFP64::from_uint(150);
    let ufp_64_8 = ~UFP64::from_uint(8);
    res = ufp_64_150 * ufp_64_8;
    assert(~UFP64::from_uint(1200) == res);

    let ufp_64_7 = ~UFP64::from_uint(7);
    let ufp_64_5 = ~UFP64::from_uint(5);
    res = ufp_64_7 * ufp_64_5;
    assert(~UFP64::from_uint(35) == res);

    true
}
