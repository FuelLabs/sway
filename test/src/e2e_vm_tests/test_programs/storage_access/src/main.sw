// For this Sway contract:

contract;

use std::constants::ETH_ID;

storage {
    x: u64 = 0,
    y: b256 = ETH_ID,
}

abi TestAbi {
    fn get_x() -> u64;
    fn get_y() -> b256;
}

impl TestAbi for Contract {
    impure fn get_x() -> u64 {
        storage.x
    }
    impure fn get_y() -> b256 {
        storage.y
    }
}


/*contract;

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
}*/
