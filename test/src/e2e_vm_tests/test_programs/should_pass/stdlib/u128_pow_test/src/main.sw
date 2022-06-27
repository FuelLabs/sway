script;

use std::assert::assert;
use std::u128::*;

fn main() -> bool {

    let one = ~U128::from(0, 1);
    let two = ~U128::from(0, 2);
    let three = ~U128::from(0, 3);

    let mut u_128: U128 = ~U128::from(0, 3);

    let mut pow_128_of_two = u_128.pow(two);
    
    assert(pow_128_of_two == ~U128::from(0, 9));

    u_128 = ~U128::from(0, 5);

    pow_128_of_two = u_128.pow(two);
    assert(pow_128_of_two == ~U128::from(0, 25));

    u_128 = ~U128::from(0, 1);

    let pow_128_of_three = u_128.pow(three);
    assert(pow_128_of_three == ~U128::from(0, 1));

    true
}
