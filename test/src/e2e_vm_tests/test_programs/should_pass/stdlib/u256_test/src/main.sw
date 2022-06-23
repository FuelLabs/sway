script;

use std::assert::assert;
use std::u256::*;

fn main() -> bool {

    let a = 11;
    let b = 42;
    let c = 101;
    let d = 69;
    let x = ~U256::from(a, b, c, d);
    assert(x.a == a);
    assert(x.b == b);
    assert(x.c == c);
    assert(x.d == d);

    true
}
