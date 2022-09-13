script;

use std::{assert::assert, ufp64::UFP64};

fn main() -> bool {
    let one = ~UFP64::from_uint(1);
    let mut res = ~UFP64::sqrt(one);
    assert(one == res);

    let ufp64_100 = ~UFP64::from_uint(100);
    res = ~UFP64::sqrt(ufp64_100);
    assert(~UFP64::from_uint(10) == res);

    let ufp64_121 = ~UFP64::from_uint(121);
    res = ~UFP64::sqrt(ufp64_121);
    assert(~UFP64::from_uint(11) == res);

    let ufp64_169 = ~UFP64::from_uint(169);
    res = ~UFP64::sqrt(ufp64_169);
    assert(~UFP64::from_uint(13) == res);

    let ufp64_49 = ~UFP64::from_uint(49);
    res = ~UFP64::sqrt(ufp64_49);
    assert(~UFP64::from_uint(7) == res);

    true
}
