contract;

use core::*;

storage {
    x: u64 = 18446744073709551615 + 1,
    y: u64 = 5 + 5,
}

abi Test {
    #[storage(read, write)]
    fn foo();
}

impl Test for Contract {
     #[storage(read, write)]
     fn foo() {
        storage.x += 1;
        storage.y += 1;
    }
}
