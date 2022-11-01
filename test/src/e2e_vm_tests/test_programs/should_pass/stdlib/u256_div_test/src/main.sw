script;

use std::assert::assert;
use std::u256::U256;
use core::num::*;

fn main() -> bool {
    let zero = U256::from(0, 0, 0, 0);
    let one = U256::from(0, 0, 0, 1);
    let two = U256::from(0, 0, 0, 2);
    let max_u64 = U256::from(0, 0, 0, u64::max());

    let div_max_two = max_u64 / two;
    assert(div_max_two.c == 0);
    assert(div_max_two.d == u64::max() >> 1);

    // Product of u64::MAX and u64::MAX.
    let mut dividend = U256::from(0, 0, u64::max(), 1);
    let mut res = dividend / max_u64;
    assert(res == U256::from(0, 0, 1, 0));

    dividend = U256::from(u64::max(), 0, 0, 0);
    let mut res = dividend / max_u64;
    assert(res == U256::from(1, 0, 0, 0));

    true
}
