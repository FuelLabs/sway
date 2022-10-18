script;

use std::{assert::assert, math::*};
use std::revert::revert;
use core::num::*;
use std::u128::*;

fn main() -> bool {
    let u_128_0: U128 = ~U128::from(0, 0);
    let u_128_1: U128 = ~U128::from(0, 1); 
    let u_128_2: U128 = ~U128::from(0, 2);
    let u_128_3: U128 = ~U128::from(0, 3);
    let u_128_6: U128 = ~U128::from(0, 6);
    let u_128_8: U128 = ~U128::from(0, 8);
    let u_128_9: U128 = ~U128::from(0, 9);
    let u_128_10: U128 = ~U128::from(0, 10);
    let u_128_100: U128 = ~U128::from(0, 100);


    assert(u_128_2.log(u_128_2) == u_128_1);    
    assert(u_128_1.log(u_128_3) == u_128_0);
    assert(u_128_8.log(u_128_2) == u_128_3);
    assert(u_128_100.log(u_128_10) == u_128_2);
    assert(u_128_100.log(u_128_2) == u_128_6);
    assert(u_128_100.log(u_128_9) == u_128_2);

    true
}
