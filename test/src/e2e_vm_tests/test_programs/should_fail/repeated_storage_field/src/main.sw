contract;

use std::constants::ZERO_B256;

storage {
    x: u64 = 0,
    x: b256 = ZERO_B256,
    x: str[4] = "0000",
}

abi Test {
    fn foo();
}

impl Test for Contract {
    fn foo() {
    }
}
