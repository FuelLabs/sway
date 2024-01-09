script;

use std::u256::*;

#[allow(deprecated)]
fn main() {
    let five = U256::from((0, 0, 0, 5));
    let two = U256::from((0, 0, 0, 2));
    assert_eq(five, two);
}