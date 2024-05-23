contract;


storage {
    x: u64 = 0,
    x: b256 = b256::zero(),
    x: str[4] = "0000",
}

abi Test {
    fn foo();
}

impl Test for Contract {
    fn foo() {
    }
}
