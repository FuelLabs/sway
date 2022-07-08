contract;

storage {
    x: u64,
}

abi Test {
    fn foo();
}

impl Test for Contract {
    fn foo() {
    }
}
