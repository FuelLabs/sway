script;

use std::assert::assert;
use std::result::*;
use std::fixed_point::*;
use std::logging::*;

fn main() -> bool {
    let zero = ~UFP64::from(0, 0);
    // let mut up = ~UFP64::from(1, 0);
    // let mut down = ~UFP64::from(2, 0);
    // let mut res = up / down;
    // assert(res == ~UFP64::from(0, 9223372036854775807));

    // up = ~UFP64::from(4, 0);
    // down = ~UFP64::from(2, 0);
    // res = up / down;

    // assert(res == ~UFP64::from(2, 0));
    
    let mut up = ~UFP64::from(9, 0);
    let mut down = ~UFP64::from(4, 0);
    let mut res = up / down;

    // log(res.value.upper);
    // log(res.value.lower);

    assert(res == ~UFP64::from(2, 4611686018427387886));

    true
}
