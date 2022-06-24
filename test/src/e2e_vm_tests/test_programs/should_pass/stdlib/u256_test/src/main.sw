script;

use core::num::*;
use std::{
    assert::assert,
    u256::*,
    result::Result,
};

fn main() -> bool {
    let new = ~U256::new();
    let (l, m, n, o) = new.into();
    assert(l == 0);
    assert(m == 0);
    assert(n == 0);
    assert(0 == 0);

    let a = 11;
    let b = 42;
    let c = 101;
    let d = 69;
    let x = ~U256::from(a, b, c, d);
    let y = ~U256::from(a, b, c, d);

    assert(x.a == a);
    assert(x.b == b);
    assert(x.c == c);
    assert(x.d == d);

    let (e, f, g, h) = x.into();
    assert(e == a);
    assert(f == b);
    assert(g == c);
    assert(h == d);

    assert(x == y);

    let max = ~U256::max();
    let err = max.to_u64();
    assert(match err {
        Result::Err(()) => {
            true
        },
        _ => {
            false
        },
    });

    let (one, two, three, four) = max.into();
    assert(one == ~u64::max());
    assert(two == ~u64::max());
    assert(three == ~u64::max());
    assert(four == ~u64::max());

    let eleven = ~U256::from(0, 0, 0, 11);
    let ok: Result<u64, ()> = eleven.to_u64();
    let unwrapped = ok.unwrap();
    assert(unwrapped == 11);

    assert(~U256::bits() == 256u32);

    true
}
