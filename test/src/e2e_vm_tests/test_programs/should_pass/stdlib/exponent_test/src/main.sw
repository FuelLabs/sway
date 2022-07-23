script;

use std::assert::assert;
use std::ufp128::*;
use std::logging::*;

fn main() -> bool {

    let mut one: UFP128 = ~UFP128::from(1, 0);
    let e = ~UFP128::exp(one);
    // assert(e == ~UFP128::from(2, 9));

    log(e.value.lower);
    log(e.value.upper);

    // let two: UFP128 = ~UFP128::from(2, 0);

    // log(two.value.lower);
    // log(two.value.upper);

    // let e_2 = ~UFP128::exp(two);
    // // assert(e == ~UFP128::from(2, 9));

    // log(e_2.value.lower);
    // log(e_2.value.upper);

    true
}
