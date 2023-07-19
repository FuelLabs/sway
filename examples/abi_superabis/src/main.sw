contract;

abi MySuperAbi {
    fn foo();
}

abi MyAbi : MySuperAbi {
    fn bar();
}

impl MySuperAbi for Contract {
    fn foo() {}
}

// The implementation of MyAbi for Contract must also implement MySuperAbi
impl MyAbi for Contract {
    fn bar() {}
}
