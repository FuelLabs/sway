contract;

mod inner;

abi Test {
    fn foo();
}

impl Test for Contract {
    fn foo() {
    }
}
