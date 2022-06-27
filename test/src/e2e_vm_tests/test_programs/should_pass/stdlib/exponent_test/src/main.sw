script;

use std::assert::assert;
use std::fixed_point::*;
use std::logging::*;

fn main() -> bool {

    let mut one: UFP64 = ~UFP64::from(1, 0);
    let e = ~UFP64::exp(one);
    // assert(e == ~UFP64::from(2, 9));

    log(e.value.lower);
    log(e.value.upper);

    // let two: UFP64 = ~UFP64::from(2, 0);

    // log(two.value.lower);
    // log(two.value.upper);

    // let e_2 = ~UFP64::exp(two);
    // // assert(e == ~UFP64::from(2, 9));

    // log(e_2.value.lower);
    // log(e_2.value.upper);

    true
}
