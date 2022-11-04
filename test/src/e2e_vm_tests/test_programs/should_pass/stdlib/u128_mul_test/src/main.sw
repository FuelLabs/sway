script;

use std::assert::assert;
use std::result::*;
use std::u128::*;

fn main() -> bool {
    let one = U128::from((0, 1));
    let two = U128::from((0, 2));
    let max_u64 = U128::from((0, u64::max()));
    let one_upper = U128::from((1, 0));

    let mul_128_of_two = max_u64 * two;
    assert(mul_128_of_two.upper == 1);
    assert(mul_128_of_two.lower == u64::max() - 1);

    let mul_128_of_four = mul_128_of_two * two;
    assert(mul_128_of_four.upper == 3);
    assert(mul_128_of_four.lower == u64::max() - 3);

    let mul_128_max = max_u64 * max_u64;
    assert(mul_128_max.upper == u64::max() - 1);
    assert(mul_128_max.lower == 1);

    true
}
