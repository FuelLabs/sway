script;

use std::u128::*;

fn main() -> bool {

    let mut u_128 = U128::from((0, 7));
    let mut pow_of_u_128 = u_128.pow(U128::from((0, 2)));
    assert(pow_of_u_128 == U128::from((0, 49)));

    pow_of_u_128 = u_128.pow(U128::from((0, 3)));
    assert(pow_of_u_128 == U128::from((0, 343)));

    u_128 = U128::from((0, 3));
    pow_of_u_128 = u_128.pow(U128::from((0, 2)));
    assert(pow_of_u_128 == U128::from((0, 9)));

    u_128 = U128::from((0, 5));
    pow_of_u_128 = u_128.pow(U128::from((0, 2)));
    assert(pow_of_u_128 == U128::from((0, 25)));

    pow_of_u_128 = u_128.pow(U128::from((0, 7)));
    assert(pow_of_u_128 == U128::from((0, 78125)));

    u_128 = U128::from((0, 8));
    pow_of_u_128 = u_128.pow(U128::from((0, 2)));
    assert(pow_of_u_128 == U128::from((0, 64)));

    pow_of_u_128 = u_128.pow(U128::from((0, 9)));
    assert(pow_of_u_128 == U128::from((0, 134217728)));

    u_128 = U128::from((0, 10));
    pow_of_u_128 = u_128.pow(U128::from((0, 2)));
    assert(pow_of_u_128 == U128::from((0, 100)));

    pow_of_u_128 = u_128.pow(U128::from((0, 5)));
    assert(pow_of_u_128 == U128::from((0, 100000)));

    u_128 = U128::from((0, 12));
    pow_of_u_128 = u_128.pow(U128::from((0, 2)));
    assert(pow_of_u_128 == U128::from((0, 144)));

    pow_of_u_128 = u_128.pow(U128::from((0, 3)));
    assert(pow_of_u_128 == U128::from((0, 1728)));

    true
}