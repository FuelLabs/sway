/*script;

use std::constants::ETH_ID;
use std::chain::assert;

struct S {
    x: u8,
    b: b256
}

fn main() {
    let x = 5;
    let y = x;
    let s1 =  S { x: 1, b: ETH_ID };
    let s = s1;
//    let mut s2 = S { x: 2, b: ETH_ID };
//    s2.b = s1.b;
//    s2.x
//    assert(s2.x == 1);
}*/

contract;

use std::constants::ETH_ID;
use std::storage::*;

struct S {
    x: u8,
    y: u8,
    b: b256
}

storage {
    number: b256 = ETH_ID,
    s: S = S { x: 0, y: 0, b: ETH_ID }  
}

const y = 999;

abi TestAbi {
    fn get_number() -> b256;
}

impl TestAbi for Contract {
    impure fn get_number() -> b256 {
        let number1 = storage.number;
        number1 
    }
}
