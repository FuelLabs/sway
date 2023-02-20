contract;

trait MyTrait {
    fn foo();
}

abi MyAbi : MyTrait {
    fn bar();
} {
    fn baz() { Self::foo() }
}

// The implementation of MyAbi for Contract should also implement MyTrait.
impl MyAbi for Contract {
    fn bar() { Self::foo() }
}

// this produces an error as of now because the compiler cannot resolve
// dependencies
impl MyTrait for Contract {
    fn foo() { }
}
