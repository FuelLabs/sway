script;

use std::assert::assert;
use std::result::*;
use std::u128::*;

fn main() -> bool {
    let one = ~U128::from(0, 1);
    let two = ~U128::from(0, 2);
    let max_u64 = ~U128::from(0, ~u64::max());
    let one_upper = ~U128::from(1, 0);

    // Product of u64::MAX and u64::MAX.
    // let dividend = ~U128::from(~u64::max() - 1, 1);
    let dividend = ~U128::from(0, ~u64::max());

    let div_max_two = dividend / two;
    assert(div_max_two.upper == 0);
    assert(div_max_two.lower == 0);
    // assert(div_max_two.lower == ~u64::max() >> 1);

    // let div_max_two = dividend / two;
    // assert(div_max_two.upper == 0);
    // assert(div_max_two.upper == dividend.upper >> 1);
    // assert(div_max_two.lower == 0);

    // let div_max_max = dividend / max_u64;
    // assert(div_max_max.upper == 0);
    // assert(div_max_max.lower == ~u64::max());

    true
}
