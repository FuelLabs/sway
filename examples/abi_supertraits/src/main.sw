contract;

trait ABIsupertrait {
    fn foo();
}

abi MyAbi : ABIsupertrait {
    fn bar();
} {
    fn baz() {
        Self::foo() // supertrait method usage
    }
}

impl ABIsupertrait for Contract {
    fn foo() {}
}

// The implementation of MyAbi for Contract must also implement ABIsupertrait
impl MyAbi for Contract {
    fn bar() {
        Self::foo() // supertrait method usage
    }
}
