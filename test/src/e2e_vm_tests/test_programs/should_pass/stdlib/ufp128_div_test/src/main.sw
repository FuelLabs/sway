script;

use std::assert::assert;
use std::result::*;
use std::ufp128::*;
use std::logging::*;

fn main() -> bool {
    let zero = ~UFP128::from(0, 0);
    let mut up = ~UFP128::from(1, 0);
    let mut down = ~UFP128::from(2, 0);
    let mut res = up / down;
    assert(res == ~UFP128::from(0, 9223372036854775807));

    up = ~UFP128::from(4, 0);
    down = ~UFP128::from(2, 0);
    res = up / down;

    assert(res == ~UFP128::from(2, 0));
    
    up = ~UFP128::from(9, 0);
    down = ~UFP128::from(4, 0);
    res = up / down;

    assert(res == ~UFP128::from(2, 4611686018427387886));

    true
}
