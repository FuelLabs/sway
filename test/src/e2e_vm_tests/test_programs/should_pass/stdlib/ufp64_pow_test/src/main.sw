script;

use std::{assert::assert, ufp64::UFP64};

fn main() -> bool {
    let one = ~UFP64::from_uint(1);
    let ufp64_1000 = ~UFP64::from_uint(1);
    let mut res = one.pow(ufp64_1000);
    assert(one == res);

    let two = ~UFP64::from_uint(2);
    let three = ~UFP64::from_uint(3);
    res = two.pow(three);
    assert(~UFP64::from_uint(8) == res);

    let ufp_64_11 = ~UFP64::from_uint(11);
    res = ufp_64_11.pow(two);
    assert(~UFP64::from_uint(121) == res);

    let five = ~UFP64::from_uint(5);
    res = five.pow(three);
    assert(~UFP64::from_uint(125) == res);

    let seven = ~UFP64::from_uint(7);
    res = seven.pow(two);
    assert(~UFP64::from_uint(49) == res);

    true
}
