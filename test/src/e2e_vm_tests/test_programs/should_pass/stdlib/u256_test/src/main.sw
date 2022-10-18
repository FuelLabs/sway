script;

use core::num::*;
use std::{assert::assert, result::Result, u256::{U256, U256Error}};

fn main() -> bool {
    // test new()
    let new = ~U256::new();
    let empty = U256 {
        a: 0,
        b: 0,
        c: 0,
        d: 0,
    };
    assert(new == empty);

    // test from() & into()
    let(l, m, n, o) = new.into();
    assert(l == 0);
    assert(m == 0);
    assert(n == 0);
    assert(o == 0);

    let a = 11;
    let b = 42;
    let c = 101;
    let d = 69;
    let x = ~U256::from(a, b, c, d);
    let y = ~U256::from(a, b, c, d);

    assert(x.a == a);
    assert(x.a != b);
    assert(x.b == b);
    assert(x.b != c);
    assert(x.c == c);
    assert(x.c != d);
    assert(x.d == d);
    assert(x.d != a);

    let(e, f, g, h) = x.into();
    assert(e == a);
    assert(f == b);
    assert(g == c);
    assert(h == d);

    assert(x == y);

    // test min() & max()
    let max = ~U256::max();
    let min = ~U256::min();
    let(one, two, three, four) = max.into();
    assert(one == ~u64::max());
    assert(two == ~u64::max());
    assert(three == ~u64::max());
    assert(four == ~u64::max());

    let(min_1, min_2, min_3, min_4) = min.into();
    assert(min_1 == ~u64::min());
    assert(min_2 == ~u64::min());
    assert(min_3 == ~u64::min());
    assert(min_4 == ~u64::min());

    // test as_u64()
    let err_1 = ~U256::from(42, 0, 0, 11).as_u64();
    assert(match err_1 {
        Result::Err(U256Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    let err_2 = ~U256::from(0, 42, 0, 11).as_u64();
    assert(match err_2 {
        Result::Err(U256Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    let err_3 = ~U256::from(0, 0, 42, 11).as_u64();
    assert(match err_3 {
        Result::Err(U256Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    let eleven = ~U256::from(0, 0, 0, 11);
    let unwrapped = eleven.as_u64().unwrap();
    assert(unwrapped == 11);

    // test as_u128()
    let err_1 = ~U256::from(42, 0, 0, 11).as_u128();
    assert(match err_1 {
        Result::Err(U256Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    let err_2 = ~U256::from(0, 42, 0, 11).as_u128();
    assert(match err_2 {
        Result::Err(U256Error::LossOfPrecision) => {
            true
        },
        _ => {
            false
        },
    });

    let elevens = ~U256::from(0, 0, 11, 11);
    let unwrapped = elevens.as_u128().unwrap();
    assert(unwrapped == ~U128::from(11, 11));

    let eleven = ~U256::from(0, 0, 0, 11);
    let unwrapped = eleven.as_u128().unwrap();
    assert(unwrapped == ~U128::from(0, 11);

    // test bits()
    assert(~U256::bits() == 256u32);

    true
}
