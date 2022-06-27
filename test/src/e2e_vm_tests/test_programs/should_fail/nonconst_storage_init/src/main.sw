contract;

use core::*;

storage {
    x: u64 = 5 + 5,
    y: u64 = 5 + 5,
}

abi Test {
    fn foo();
}

impl Test for Contract {
    fn foo() {
        storage.x += 1;
        storage.y += 1;
    }
}
