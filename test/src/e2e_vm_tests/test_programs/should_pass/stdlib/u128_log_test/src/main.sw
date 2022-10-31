script;

use std::math::*;
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
    let u_128_21: U128 = ~U128::from(0, 21);
    let u_128_42: U128 = ~U128::from(0, 42);
    let u_128_64: U128 = ~U128::from(0, 64);
    let u_128_100: U128 = ~U128::from(0, 100);
    let u_128_128: U128 = ~U128::from(0, 128);
    let u_128_max_div_2: U128 = ~U128::from(1, 0);
    let u_128_max: U128 = ~U128::max();


    assert(u_128_2.log(u_128_2) == u_128_1);    
    assert(u_128_1.log(u_128_3) == u_128_0);
    assert(u_128_8.log(u_128_2) == u_128_3);
    assert(u_128_100.log(u_128_10) == u_128_2);
    assert(u_128_100.log(u_128_2) == u_128_6);
    assert(u_128_100.log(u_128_9) == u_128_2);
    assert(u_128_max.log(u_128_2) == u_128_128);
    assert(u_128_max.log(u_128_9) == u_128_42);
    assert(u_128_max_div_2.log(u_128_2) == u_128_64);
    assert(u_128_max_div_2.log(u_128_9) == u_128_21);

    true
}
